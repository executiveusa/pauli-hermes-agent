#!/usr/bin/env python3
"""Deterministic read-only factory stage selector. Never invokes external tools."""
import argparse
import json
from pathlib import Path

STAGES = [
 ("00","intake",["vibe:w1-intake-to-spec"]),
 ("01","inventory",["vibe:w7-research-and-skill-making"]),
 ("02","proven_better_new",["proven-better-new"]),
 ("03","discovery",["pm-skills-navigator"]),
 ("04","challenge",["thinking-expert"]),
 ("05","yc_market",["yc-startup-fundamentals"]),
 ("06","commercial",["pricing-strategist"]),
 ("07","presell",["vibe:w6-copy-and-voice"]),
 ("08","spec",["vibe:w1-intake-to-spec"]),
 ("09","build",["vibe:w2-build"]),
 ("10","proof",["vibe:w3-proof-and-review"]),
 ("11","launch",["vibe:w4-ship-and-operate"]),
 ("12","measure",["vibe:w4-ship-and-operate"]),
]
NAMES = [row[1] for row in STAGES]
BOOL_FIELDS = ('owner_verified','market_research_available','commercial_evidence',
               'outreach_authorized','build_authorized','independent_reviewer','release_authorized')
def route(data):
    if not isinstance(data, dict): raise ValueError("object input required")
    for key in BOOL_FIELDS:
        if key in data and type(data[key]) is not bool:
            raise ValueError(f"{key} must be a JSON boolean, never a string or number")
    passed = data.get("passed_stages", [])
    if not isinstance(passed,list) or any(not isinstance(x,str) for x in passed) or len(passed)!=len(set(passed)) or any(x not in NAMES for x in passed):
        raise ValueError("invalid passed_stages")
    if passed != NAMES[:len(passed)]:
        raise ValueError("out-of-order PASS claims")
    mode = data.get("mode","GREENFIELD")
    if mode not in ("GREENFIELD","BROWNFIELD","RESCUE"): raise ValueError("invalid mode")
    if len(passed)==len(NAMES):
        return {"status":"COMPLETE_FOR_THIS_CYCLE","next":"new evidence-based iteration","actions_performed":[]}
    number, name, skill = STAGES[len(passed)]
    domain=data.get("domain","general")
    # Skills are exact names verified installed locally on Hermes.
    # Domain-specific upstream variants remain references, never an implicit install.
    optional_references=[]
    if name=="yc_market":
        special={"b2b-ai":"b2b-ai-startup-levie","devtools":"developer-tools-strategy-truell","ai-product":"ai-product-building-heller"}
        if domain in special: optional_references=["jona:"+special[domain]]
    elif name=="challenge" and domain in ("devtools","infrastructure"):
        optional_references=["felixgeelhaar:engineering-expert"]
    elif name=="commercial":
        optional_references=["alirezarezvani:financial-analyst","alirezarezvani:commercial-forecaster"]
    hold=None
    if not str(data.get("product","")).strip(): hold="missing product/idea"
    elif name=="intake" and data.get("owner_verified") is not True: hold="ownership/authority not verified"
    elif name=="proven_better_new" and data.get("market_research_available") is not True: hold="market research unavailable: label UNRESEARCHED"
    elif name=="commercial" and data.get("commercial_evidence") is not True: hold="pricing/buyer/cost/30-day path evidence missing"
    elif name=="presell" and data.get("outreach_authorized") is not True: hold="outreach not authorized; research and drafts only"
    elif name=="build" and data.get("build_authorized") is not True: hold="build not authorized"
    elif name=="proof" and data.get("independent_reviewer") is not True: hold="independent reviewer absent"
    elif name=="launch" and data.get("release_authorized") is not True: hold="release not authorized"
    return {"status":"HOLD" if hold else "READY_FOR_REVIEW",
            "stage":number,"stage_name":name,"mode":mode,"skills":skill,
            "optional_references":optional_references,
            "blocked_reason":hold,"actions_performed":[],
            "note":"Stage proposal only; check actual source/skill availability and evidence."}
def main():
    p=argparse.ArgumentParser();p.add_argument("--input",required=True);args=p.parse_args()
    try: print(json.dumps(route(json.loads(Path(args.input).read_text(encoding="utf-8"))),indent=2))
    except (ValueError,OSError) as e: p.error(str(e))
if __name__=="__main__": main()
