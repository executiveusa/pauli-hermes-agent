#!/usr/bin/env python3
"""Karpathy-style LLM Council for Hermes/Cosmos.

Three stages:
1. Independent first opinions from multiple frontier models.
2. Anonymous peer review/ranking of those opinions.
3. Chairman synthesis over the opinions and peer-review evidence.

This deliberately reuses Hermes' existing OpenRouter/Mixture-of-Agents client
rather than introducing another provider stack.
"""

from __future__ import annotations

import asyncio
import json
import logging
import re
import secrets
import time
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Sequence, Tuple

from tools.mixture_of_agents_tool import (
    AGGREGATOR_MODEL,
    REFERENCE_MODELS,
    _run_aggregator_model,
    _run_reference_model_safe,
    check_moa_requirements,
)
from tools.registry import registry

logger = logging.getLogger(__name__)

_RANKING_RE = re.compile(
    r"(?:FINAL\s+RANKING|RANKING)\s*:?\s*([A-Z](?:\s*(?:>|,|\n|-)\s*[A-Z])*)",
    re.IGNORECASE,
)


@dataclass(frozen=True)
class CouncilOpinion:
    model: str
    response: str


def _labels(count: int) -> List[str]:
    if count > 26:
        raise ValueError("LLM Council supports at most 26 members")
    return [chr(ord("A") + i) for i in range(count)]


def _parse_ranking(text: str, valid_labels: Sequence[str]) -> List[str]:
    """Parse a reviewer ranking while tolerating prose around the final list."""
    valid = {label.upper() for label in valid_labels}
    match = _RANKING_RE.search(text or "")
    candidate = match.group(1) if match else text
    seen: set[str] = set()
    parsed: List[str] = []
    for token in re.findall(r"\b([A-Z])\b", candidate.upper()):
        if token in valid and token not in seen:
            parsed.append(token)
            seen.add(token)
    return parsed


def _borda(rankings: Sequence[Sequence[int]], count: int) -> List[Tuple[int, int]]:
    scores = [0] * count
    for ranking in rankings:
        width = len(ranking)
        for position, response_index in enumerate(ranking):
            if 0 <= response_index < count:
                scores[response_index] += max(width - position - 1, 0)
    return sorted(enumerate(scores), key=lambda item: (-item[1], item[0]))


def _review_prompt(user_prompt: str, labeled_responses: Sequence[Tuple[str, str]]) -> str:
    responses = "\n\n".join(
        f"RESPONSE {label}:\n{response}" for label, response in labeled_responses
    )
    return f"""You are a peer reviewer in an anonymous LLM Council.

Original question:
{user_prompt}

Below are anonymous candidate answers. Their model identities are intentionally
hidden. Evaluate accuracy, insight, completeness, practical usefulness, and
unsupported assumptions. Do not infer or speculate about authorship.

{responses}

Return:
1. A concise critique of the strongest and weakest aspects.
2. Important disagreements or blind spots.
3. A final best-to-worst ordering on one line exactly like:
FINAL RANKING: A > C > B

Rank every response exactly once."""


def _chairman_prompt(
    user_prompt: str,
    opinions: Sequence[CouncilOpinion],
    reviews: Sequence[Dict[str, Any]],
    aggregate: Sequence[Tuple[int, int]],
) -> str:
    opinion_text = "\n\n".join(
        f"OPINION {idx + 1}:\n{opinion.response}" for idx, opinion in enumerate(opinions)
    )
    review_text = "\n\n".join(
        f"PEER REVIEW {idx + 1}:\n{review['review']}" for idx, review in enumerate(reviews)
    )
    rank_text = ", ".join(
        f"Opinion {idx + 1}={score}" for idx, score in aggregate
    )
    return f"""You are the Chairman of an LLM Council.

Original question:
{user_prompt}

STAGE 1 — INDEPENDENT OPINIONS
{opinion_text}

STAGE 2 — ANONYMOUS PEER REVIEWS
{review_text}

Aggregate Borda ordering/scores:
{rank_text}

Synthesize a single final answer. Do not merely vote or concatenate. Resolve
conflicts using evidence and reasoning. Preserve useful minority viewpoints when
they expose a genuine risk or uncertainty. Clearly distinguish consensus,
disagreement, assumptions, and the recommended conclusion when appropriate.
"""


async def llm_council_tool(
    user_prompt: str,
    council_models: Optional[List[str]] = None,
    chairman_model: Optional[str] = None,
) -> str:
    """Run independent answers -> anonymous peer review -> chairman synthesis."""
    started = time.monotonic()
    prompt = (user_prompt or "").strip()
    if not prompt:
        return json.dumps({"success": False, "error": "user_prompt is required"})

    models = list(dict.fromkeys(council_models or REFERENCE_MODELS))
    if len(models) < 2:
        return json.dumps({"success": False, "error": "LLM Council requires at least two distinct models"})
    if len(models) > 8:
        return json.dumps({"success": False, "error": "LLM Council is capped at eight models per run"})

    # Stage 1: independent first opinions. No model sees another answer here.
    first_pass = await asyncio.gather(
        *[_run_reference_model_safe(model, prompt) for model in models]
    )
    opinions = [
        CouncilOpinion(model=model, response=response)
        for model, response, ok in first_pass
        if ok and response.strip()
    ]
    failures = [model for model, _response, ok in first_pass if not ok]
    if len(opinions) < 2:
        return json.dumps(
            {
                "success": False,
                "error": "Fewer than two council members returned usable first opinions",
                "failed_models": failures,
            },
            indent=2,
        )

    # Stage 2: every reviewer sees anonymized/shuffled answers. We keep a private
    # per-reviewer label map so rank aggregation can recover canonical indices.
    review_jobs = []
    review_maps: List[Dict[str, int]] = []
    canonical = list(range(len(opinions)))
    for reviewer in opinions:
        shuffled = canonical.copy()
        secrets.SystemRandom().shuffle(shuffled)
        labels = _labels(len(shuffled))
        label_map = {label: canonical_index for label, canonical_index in zip(labels, shuffled)}
        review_maps.append(label_map)
        labeled = [(label, opinions[index].response) for label, index in zip(labels, shuffled)]
        review_jobs.append(
            _run_reference_model_safe(reviewer.model, _review_prompt(prompt, labeled))
        )

    raw_reviews = await asyncio.gather(*review_jobs)
    reviews: List[Dict[str, Any]] = []
    canonical_rankings: List[List[int]] = []
    for idx, (reviewer_model, review_text, ok) in enumerate(raw_reviews):
        if not ok or not review_text.strip():
            failures.append(reviewer_model)
            continue
        label_map = review_maps[idx]
        label_ranking = _parse_ranking(review_text, list(label_map))
        ranking = [label_map[label] for label in label_ranking if label in label_map]
        # If a reviewer fails to format every label, append missing items without
        # granting artificial preference to any omitted response.
        ranking.extend(index for index in canonical if index not in ranking)
        canonical_rankings.append(ranking)
        reviews.append(
            {
                "reviewer": reviewer_model,
                "review": review_text,
                "ranking": [opinions[index].model for index in ranking],
            }
        )

    aggregate = _borda(canonical_rankings, len(opinions))
    chairman = chairman_model or AGGREGATOR_MODEL

    # Reuse the existing hardened aggregator transport. When a non-default
    # chairman is requested, call the same OpenRouter helper by temporarily
    # passing the desired model through a small local request path.
    if chairman == AGGREGATOR_MODEL:
        final_response = await _run_aggregator_model(
            "You are the Chairman of an LLM Council. Produce the final synthesis.",
            _chairman_prompt(prompt, opinions, reviews, aggregate),
        )
    else:
        _model, final_response, ok = await _run_reference_model_safe(
            chairman,
            _chairman_prompt(prompt, opinions, reviews, aggregate),
            temperature=0.4,
        )
        if not ok:
            return json.dumps(
                {
                    "success": False,
                    "error": f"Chairman model {chairman} failed",
                    "failed_models": failures + [chairman],
                },
                indent=2,
            )

    ordered_models = [opinions[index].model for index, _score in aggregate]
    score_by_model = {opinions[index].model: score for index, score in aggregate}

    return json.dumps(
        {
            "success": True,
            "response": final_response,
            "council": {
                "method": "independent_answers_anonymous_peer_review_chairman",
                "members": [opinion.model for opinion in opinions],
                "chairman": chairman,
                "aggregate_ranking": ordered_models,
                "scores": score_by_model,
                "failed_models": sorted(set(failures)),
            },
            "stage1": [
                {"model": opinion.model, "response": opinion.response}
                for opinion in opinions
            ],
            "stage2": reviews,
            "processing_time_seconds": round(time.monotonic() - started, 3),
        },
        indent=2,
        ensure_ascii=False,
    )


LLM_COUNCIL_SCHEMA = {
    "name": "llm_council",
    "description": (
        "Convene a high-reasoning LLM Council: independent first opinions, "
        "anonymous peer review/ranking, then Chairman synthesis. Use when the "
        "owner explicitly asks for the council or when a consequential decision "
        "benefits from diverse model judgment. More expensive than one model."
    ),
    "parameters": {
        "type": "object",
        "properties": {
            "user_prompt": {
                "type": "string",
                "description": "The question or decision for the council.",
            },
            "council_models": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Optional 2-8 OpenRouter model identifiers. Defaults to the existing Hermes MoA roster.",
            },
            "chairman_model": {
                "type": "string",
                "description": "Optional OpenRouter model identifier for final synthesis.",
            },
        },
        "required": ["user_prompt"],
    },
}

registry.register(
    name="llm_council",
    toolset="moa",
    schema=LLM_COUNCIL_SCHEMA,
    handler=lambda args, **kw: llm_council_tool(
        user_prompt=args.get("user_prompt", ""),
        council_models=args.get("council_models"),
        chairman_model=args.get("chairman_model"),
    ),
    check_fn=check_moa_requirements,
    requires_env=["OPENROUTER_API_KEY"],
    is_async=True,
    emoji="🏛️",
)
