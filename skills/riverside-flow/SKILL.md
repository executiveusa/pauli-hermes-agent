---
name: riverside-flow
description: ICM-governed Riverside recording, interview, fundraising, relationship, repurposing, narrative-control, distribution, A/B testing, analytics, and learning workflow. Use when planning or processing Riverside podcasts, interviews, webinars, donor conversations, founder stories, youth/community stories, documentary interviews, or Riverside-powered client content systems.
---

# Riverside Flow

## Purpose

Turn one high-value conversation into durable media, measurable relationships, fundraising/sales opportunities, and reusable intelligence without fabricating claims or losing narrative control.

Riverside is the capture and first-pass repurposing layer. Hermes is the orchestrator. Existing media, social, fundraising, CRM, and publishing skills remain the system of record for their domains.

## Canonical ICM

Load only the files needed for the current stage from `skills/riverside-flow/ICM/`.

Execution sequence:

`00_intake -> 01_guest_strategy -> 02_record -> 03_extract -> 04_narrative -> 05_repurpose -> 06_distribute -> 07_ab_test -> 08_learn -> 09_relationship_followup`

## Walk test

Before acting, Hermes must be able to answer:

1. What measurable outcome is this conversation supposed to create?
2. Who is the guest/audience and why are they strategically relevant?
3. What is the single primary CTA?
4. What facts, claims, names, consent limits, and sensitive material are locked?
5. What assets are allowed to be generated from the recording?
6. Which channel(s) are being tested and what is the A/B hypothesis?
7. What proof will count as success at 24h, 72h, 7d, and 30d?
8. What relationship follow-up is due after the episode?

If any answer is unknown, mark it UNKNOWN and continue only with reversible prep work.

## Hard gates

- Never expose API keys or credentials.
- Never claim Riverside API access unless verified on the active account/plan.
- Never publish a guest clip, quote, testimonial, donor statement, youth story, or sensitive interview segment outside the consent boundary.
- Never invent outcomes, donations, partnerships, impact, credentials, quotes, or endorsements.
- Preserve the factual spine across all A/B variants; test packaging, not truth.
- Builder cannot approve itself for public release.
- Public publishing requires exact-artifact approval unless an existing governed auto-publish policy explicitly allows that destination.
- Use owned/licensed/approved media only.
- For minors or youth stories, default to stricter consent and privacy handling.
- Store source transcript, source recording IDs, timestamps, and derivative lineage where practical.
- Measure business/fundraising outcomes in addition to reach.

## Primary operating model

One conversation should attempt to produce:

- 1 canonical long-form episode or interview
- 3-10 short-form clips
- 1 article or newsletter draft
- 3-5 social posts
- 1 donor/client/partner follow-up path
- 1 structured relationship/opportunity record
- 1 A/B test with a single hypothesis
- 24h / 72h / 7d learning receipts

## Narrative control

Hermes must maintain a canonical `Narrative Spine` before producing variants:

- What happened / what is being discussed
- Why it matters
- Verified evidence
- What is still uncertain
- What the guest actually said
- What the organization is allowed to claim
- Desired audience interpretation
- Primary CTA
- Claims that must never be implied

Variants may change hook, order, title, thumbnail, framing, caption density, duration, CTA wording, and platform packaging. They may not change the factual meaning.

## Routing

- Fundraising/donor context -> `new-world-kids-fundraising` when applicable.
- Social distribution -> `scroll-media-operator`.
- Interactive derivatives -> `social-drop-factory`.
- Documentary/cinematic edit -> `cinematic-master-editor`.
- Adversarial idea discovery -> `black-swan-skills`.
- Quality/taste review -> `gauntlet-loop`.

## Default commercial product

**Founder / Impact Story Sprint**

One 45-90 minute Riverside session -> canonical story -> long-form video -> short clips -> article/newsletter -> social package -> CTA -> analytics -> follow-up opportunities.

Do not build new infrastructure to prove this offer. Use Riverside + existing Hermes skills first.

## Required major-work report

Return:

DECISION
CHANGES
PROOF
STATUS
COMMERCIAL IMPACT
RISKS
ROLLBACK
NEXT
HUMAN APPROVAL
