#!/usr/bin/env python3
"""Durable recovery, safe retries, bounded dispatch, and context validation."""
from __future__ import annotations
import argparse, json, os, sys, tempfile
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any

V="1.0.0"; HARD_CAP=8
ACTIVE={"dispatched","running"}; READY={"pending","ready","retry_scheduled"}
EFFECTS={"none","local_reversible","approval_required","irreversible"}
OUTCOMES={"not_applied","applied","rolled_back","partially_applied","unknown"}

def now(): return datetime.now(timezone.utc)
def iso(): return now().isoformat()
def parse(v): return datetime.fromisoformat(str(v).replace("Z","+00:00"))
def load(p: Path):
    with p.open(encoding="utf-8") as f: x=json.load(f)
    if not isinstance(x,dict): raise ValueError(f"Expected object: {p}")
    return x
def save(p: Path,x: dict):
    p.parent.mkdir(parents=True,exist_ok=True); fd,t=tempfile.mkstemp(prefix=f".{p.name}.",suffix=".tmp",dir=p.parent)
    try:
        with os.fdopen(fd,"w",encoding="utf-8") as f:
            json.dump(x,f,indent=2,sort_keys=True,ensure_ascii=False); f.write("\n"); f.flush(); os.fsync(f.fileno())
        os.replace(t,p)
    except Exception:
        try: os.unlink(t)
        except FileNotFoundError: pass
        raise
def event(run: Path,kind: str,data: dict):
    with (run/"events.jsonl").open("a",encoding="utf-8") as f:
        f.write(json.dumps({"at":iso(),"type":kind,"payload":data},sort_keys=True)+"\n"); f.flush(); os.fsync(f.fileno())
def taskmap(m): return {t["id"]:t for t in m["tasks"]}
def defaults(s,m):
    s.setdefault("usage",{}).setdefault("reserved_cost_usd",0.0); s["usage"].setdefault("estimated_cost_usd",0.0)
    s.setdefault("providers",{}); s.setdefault("pending_approvals",[]); s.setdefault("unresolved",[])
    s.setdefault("adaptive_concurrency",min(int(m["limits"]["max_concurrent_children"]),HARD_CAP))
def validate_limits(m):
    c=int(m["limits"]["max_concurrent_children"]); p=int(m["limits"].get("provider_concurrency_cap",c))
    if not 1<=c<=HARD_CAP: raise ValueError(f"max_concurrent_children must be between 1 and {HARD_CAP}")
    if not 1<=p<=HARD_CAP: raise ValueError(f"provider_concurrency_cap must be between 1 and {HARD_CAP}")
def contract_errors(t):
    e=[]
    for k in ("id","title","goal","context","side_effect_class"):
        if not isinstance(t.get(k),str) or not t[k].strip(): e.append(f"{k} must be non-empty")
    if t.get("side_effect_class") not in EFFECTS: e.append("invalid side_effect_class")
    for k in ("dependencies","acceptance_criteria","required_evidence","allowed_inputs","allowed_outputs","prohibited_actions"):
        if not isinstance(t.get(k),list): e.append(f"{k} must be an array")
        elif k!="dependencies" and k!="prohibited_actions" and not t[k]: e.append(f"{k} must not be empty")
    return e
def deps_done(t,s): return all(s["tasks"][d]["status"]=="completed" for d in t.get("dependencies",[]))
def approval(run,mid,t):
    if t["side_effect_class"] in {"none","local_reversible"}: return True
    if t["side_effect_class"]=="irreversible": return False
    p=run/"approvals"/f"{t['id']}.json"
    if not p.exists(): return False
    a=load(p); exp=a.get("expires_at")
    return a.get("mission_id")==mid and a.get("task_id")==t["id"] and a.get("approved") is True and (not exp or parse(exp)>now())
def provider(s,name,cap):
    p=s["providers"].setdefault(name,{"concurrency_limit":cap,"cooldown_until":None,"rate_limit_events":0})
    return p
def attempt_path(run,tid,n): return run/"attempts"/tid/f"attempt-{n:03d}"

def recover(run,m,s):
    tm=taskmap(m); out=[]
    for tid,st in s["tasks"].items():
        if st.get("status") not in ACTIVE: continue
        rel=st.get("active_attempt_state")
        if not rel or not (run/rel).exists(): st.update(status="unknown",last_error="Missing attempt state"); out.append({"task_id":tid,"action":"unknown"}); continue
        ap=run/rel; a=load(ap)
        if (ap.parent/"result.json").exists(): out.append({"task_id":tid,"action":"result_present"}); continue
        if parse(a["lease_expires_at"])>now(): continue
        cp=a.get("latest_checkpoint"); effect=tm[tid]["side_effect_class"]
        st.update(active_attempt_state=None,last_error="Worker lease expired",updated_at=iso())
        if effect=="none": st.update(status="retry_scheduled",resume_from=cp); out.append({"task_id":tid,"action":"resume","checkpoint":cp})
        elif effect=="local_reversible" and cp and (run/cp).exists() and load(run/cp).get("local_state_consistent") is True:
            st.update(status="retry_scheduled",resume_from=cp); out.append({"task_id":tid,"action":"resume_local","checkpoint":cp})
        else:
            rp=ap.parent/"side-effect-receipt.json"
            if rp.exists() and load(rp).get("outcome") in {"not_applied","rolled_back"}:
                st.update(status="retry_scheduled",resume_from=cp); out.append({"task_id":tid,"action":"retry_after_receipt"})
            else:
                st["status"]="needs_reconciliation"; out.append({"task_id":tid,"action":"reconcile_required"})
                save(ap.parent/"reconciliation-required.json",{"schema_version":V,"mission_id":m["mission_id"],"task_id":tid,"attempt":a["attempt"],"idempotency_key":a["idempotency_key"],"created_at":iso()})
    return out

def packet(run,m,t,st,n,ttl):
    out=attempt_path(run,t["id"],n); resume=st.get("resume_from")
    x={"schema_version":V,"mission_id":m["mission_id"],"mission_objective":m["objective"],"mission_constraints":m.get("constraints",[]),
       "task_id":t["id"],"title":t["title"],"goal":t["goal"],"context":t["context"],"dependencies":t.get("dependencies",[]),
       "acceptance_criteria":t["acceptance_criteria"],"required_evidence":t["required_evidence"],"allowed_inputs":t["allowed_inputs"],
       "allowed_outputs":t["allowed_outputs"],"prohibited_actions":t["prohibited_actions"],"side_effect_class":t["side_effect_class"],
       "attempt":n,"idempotency_key":f"{m['mission_id']}:{t['id']}:{n}","run_directory":str(run.resolve()),
       "result_path":str((out/"result.json").resolve()),"checkpoint_directory":str((out/"checkpoints").resolve()),
       "side_effect_journal_path":str((out/"side-effects.jsonl").resolve()),"resume_from":str((run/resume).resolve()) if resume else None,
       "limits":{"max_iterations":int(t.get("max_iterations",m["limits"]["max_iterations_per_child"])),"lease_seconds":ttl,"estimated_cost_usd":float(t.get("estimated_cost_usd",.1))},
       "worker_requirements":["Checkpoint after each completed unit","Journal side effects before and after execution","Atomically write result JSON before returning","Never edit shared mission state"]}
    needed=("mission_id","task_id","goal","context","acceptance_criteria","required_evidence","allowed_inputs","allowed_outputs","result_path","checkpoint_directory")
    missing=[k for k in needed if not x.get(k)]
    if missing: raise ValueError(f"Generated incomplete context packet for {t['id']}: {missing}")
    return x

def cmd_prepare(a):
    run=Path(a.run).expanduser().resolve(); m=load(run/"mission.json"); s=load(run/"state.json"); validate_limits(m); defaults(s,m)
    rec=recover(run,m,s)
    if any(v.get("status") in {"needs_reconciliation","unknown","blocked","failed"} for v in s["tasks"].values()):
        s["status"]="blocked"; save(run/"state.json",s); print(json.dumps({"dispatch":[],"reason":"reconciliation_required","recovered":rec},indent=2)); return 3
    candidates=[t for t in m["tasks"] if s["tasks"][t["id"]]["status"] in READY and deps_done(t,s)]
    errs=[]
    for t in candidates:
        e=contract_errors(t)
        if e: errs.append(f"{t['id']}: {'; '.join(e)}")
    if errs:
        s["status"]="blocked"; s["unresolved"]=sorted(set(s["unresolved"]+errs)); save(run/"state.json",s); event(run,"context_packet_validation_failed",{"errors":errs}); print(json.dumps({"dispatch":[],"reason":"invalid_context_packet","errors":errs},indent=2)); return 4
    conf=int(m["limits"]["max_concurrent_children"]); pcap=int(m["limits"].get("provider_concurrency_cap",conf)); adaptive=min(conf,int(s["adaptive_concurrency"]),HARD_CAP)
    active=sum(v.get("status") in ACTIVE for v in s["tasks"].values()); slots=max(0,adaptive-active)
    total=float(m["limits"].get("max_total_cost_usd",0)); used=float(s["usage"].get("estimated_cost_usd",0))+float(s["usage"].get("reserved_cost_usd",0)); rem=max(0,total-used) if total else float("inf")
    budget=min(rem,float(m["limits"].get("max_dispatch_cost_usd",rem))); chosen=[]; cost=0.; pending=[]; tm=taskmap(m)
    for t in sorted(candidates,key=lambda x:(int(x.get("priority",100)),x["id"])):
        if len(chosen)>=slots: break
        if not approval(run,m["mission_id"],t): pending.append(t["id"]); continue
        name=str(t.get("provider","default")); ps=provider(s,name,pcap); cool=ps.get("cooldown_until")
        if cool and parse(cool)>now(): continue
        activep=sum(v.get("status") in ACTIVE and tm[k].get("provider","default")==name for k,v in s["tasks"].items())
        selectedp=sum(x[0].get("provider","default")==name for x in chosen)
        if activep+selectedp>=min(int(ps["concurrency_limit"]),pcap,HARD_CAP): continue
        est=float(t.get("estimated_cost_usd",.1))
        if cost+est>budget+1e-9: continue
        chosen.append((t,est)); cost+=est
    dispatch=[]
    for t,est in chosen:
        st=s["tasks"][t["id"]]; n=int(st.get("attempts",0))+1
        if n>int(st.get("max_attempts",m["limits"]["max_attempts_per_task"])): st.update(status="failed",last_error="Attempt budget exhausted"); continue
        p=packet(run,m,t,st,n,a.worker_ttl_seconds); d=attempt_path(run,t["id"],n); save(d/"request.json",p)
        ast={"schema_version":V,"mission_id":m["mission_id"],"task_id":t["id"],"attempt":n,"status":"running","idempotency_key":p["idempotency_key"],"side_effect_class":t["side_effect_class"],"worker_id":None,"created_at":iso(),"heartbeat_at":iso(),"lease_expires_at":(now()+timedelta(seconds=a.worker_ttl_seconds)).isoformat(),"latest_checkpoint":st.get("resume_from"),"request_path":str((d/"request.json").relative_to(run))}
        save(d/"attempt-state.json",ast); st.update(status="dispatched",attempts=n,active_attempt_state=str((d/"attempt-state.json").relative_to(run)),request_path=str((d/"request.json").relative_to(run)),last_error=None,updated_at=iso())
        dispatch.append({"task_id":t["id"],"attempt":n,"request_path":str(d/"request.json"),"result_path":p["result_path"],"max_iterations":p["limits"]["max_iterations"],"estimated_cost_usd":est,"resume_from":p["resume_from"]})
    s["pending_approvals"]=sorted(set(pending)); s["usage"]["reserved_cost_usd"]=round(float(s["usage"]["reserved_cost_usd"])+cost,6)
    s["status"]="waiting_workers" if dispatch else ("waiting_approval" if pending else ("blocked" if candidates and budget<=0 else "ready")); s["updated_at"]=iso(); save(run/"state.json",s)
    event(run,"batch_prepared",{"dispatch_count":len(dispatch),"adaptive_concurrency":adaptive,"reserved_cost_usd":cost,"pending_approvals":pending,"recovered":rec})
    print(json.dumps({"dispatch":dispatch,"pending_approvals":pending,"recovered":rec},indent=2)); return 0

def cmd_heartbeat(a):
    run=Path(a.run).resolve(); s=load(run/"state.json"); st=s["tasks"][a.task_id]; rel=st.get("active_attempt_state")
    if not rel: raise RuntimeError("Task has no active attempt")
    ap=run/rel; x=load(ap)
    if int(x["attempt"])!=a.attempt: raise RuntimeError("Attempt mismatch")
    x.update(status="running",worker_id=a.worker_id,heartbeat_at=iso(),lease_expires_at=(now()+timedelta(seconds=a.worker_ttl_seconds)).isoformat())
    cp_rel=None
    if a.checkpoint_file:
        cp=load(Path(a.checkpoint_file).resolve()); req=("schema_version","mission_id","task_id","attempt","completed_units","next_unit","artifacts","side_effects","local_state_consistent")
        miss=[k for k in req if k not in cp]
        if miss or cp.get("mission_id")!=s["mission_id"] or cp.get("task_id")!=a.task_id or int(cp.get("attempt",-1))!=a.attempt: raise ValueError(f"Invalid checkpoint: {miss or 'identity mismatch'}")
        cdir=ap.parent/"checkpoints"; n=len(list(cdir.glob("checkpoint-*.json")))+1 if cdir.exists() else 1; dst=cdir/f"checkpoint-{n:04d}.json"; cp["saved_at"]=iso(); save(dst,cp); cp_rel=str(dst.relative_to(run)); x["latest_checkpoint"]=cp_rel; st["resume_from"]=cp_rel
    save(ap,x); st.update(status="running",updated_at=iso()); save(run/"state.json",s); event(run,"worker_heartbeat",{"task_id":a.task_id,"attempt":a.attempt,"checkpoint":cp_rel}); print(json.dumps({"heartbeat":"accepted","checkpoint":cp_rel},indent=2)); return 0

def cmd_recover(a):
    run=Path(a.run).resolve(); m=load(run/"mission.json"); s=load(run/"state.json"); defaults(s,m); r=recover(run,m,s); b=[k for k,v in s["tasks"].items() if v.get("status") in {"needs_reconciliation","unknown","blocked","failed"}]; s["status"]="blocked" if b else "ready"; s["updated_at"]=iso(); save(run/"state.json",s); event(run,"orphan_recovery_completed",{"recovered":r,"blocking":b}); print(json.dumps({"recovered":r,"blocking":b,"status":s["status"]},indent=2)); return 0

def cmd_receipt(a):
    run=Path(a.run).resolve(); m=load(run/"mission.json"); s=load(run/"state.json"); st=s["tasks"][a.task_id]; d=attempt_path(run,a.task_id,a.attempt); ast=load(d/"attempt-state.json"); r=load(Path(a.receipt_file).resolve())
    req=("schema_version","mission_id","task_id","attempt","idempotency_key","outcome","verified_at","verified_by","evidence"); miss=[k for k in req if k not in r]
    if miss or r.get("mission_id")!=m["mission_id"] or r.get("task_id")!=a.task_id or int(r.get("attempt",-1))!=a.attempt or r.get("idempotency_key")!=ast["idempotency_key"] or r.get("outcome") not in OUTCOMES or not r.get("evidence"): raise ValueError(f"Invalid side-effect receipt: {miss or 'identity/outcome/evidence'}")
    save(d/"side-effect-receipt.json",r); o=r["outcome"]
    if o in {"not_applied","rolled_back"}: st.update(status="retry_scheduled",active_attempt_state=None,last_error=f"Reconciled as {o}; retry permitted"); s["status"]="ready"
    elif o=="applied": st["status"]="needs_review"; s["status"]="blocked"; s["unresolved"]=sorted(set(s["unresolved"]+[f"Review applied effect for {a.task_id}"]))
    else: st["status"]="needs_reconciliation"; s["status"]="blocked"; s["unresolved"]=sorted(set(s["unresolved"]+[f"Effect remains {o} for {a.task_id}"]))
    st["updated_at"]=iso(); s["updated_at"]=iso(); save(run/"state.json",s); event(run,"side_effect_reconciled",{"task_id":a.task_id,"attempt":a.attempt,"outcome":o}); print(json.dumps({"task_id":a.task_id,"outcome":o,"task_status":st["status"],"mission_status":s["status"]},indent=2)); return 0

def cmd_rate(a):
    run=Path(a.run).resolve(); m=load(run/"mission.json"); s=load(run/"state.json"); validate_limits(m); defaults(s,m); cap=int(m["limits"].get("provider_concurrency_cap",m["limits"]["max_concurrent_children"])); p=provider(s,a.provider,cap); p["rate_limit_events"]=int(p.get("rate_limit_events",0))+1; p["concurrency_limit"]=max(1,int(p["concurrency_limit"])//2); p["cooldown_until"]=(now()+timedelta(seconds=a.retry_after_seconds)).isoformat(); s["adaptive_concurrency"]=max(1,int(s["adaptive_concurrency"])//2); s["updated_at"]=iso(); save(run/"state.json",s); event(run,"provider_rate_limited",{"provider":a.provider,"retry_after_seconds":a.retry_after_seconds,"concurrency_limit":p["concurrency_limit"]}); print(json.dumps({"provider":a.provider,**p,"adaptive_concurrency":s["adaptive_concurrency"]},indent=2)); return 0

def parser():
    p=argparse.ArgumentParser(); sub=p.add_subparsers(dest="command",required=True)
    q=sub.add_parser("prepare-batch"); q.add_argument("--run",required=True); q.add_argument("--controller",required=True); q.add_argument("--worker-ttl-seconds",type=int,default=900); q.set_defaults(func=cmd_prepare)
    q=sub.add_parser("heartbeat"); q.add_argument("--run",required=True); q.add_argument("--task-id",required=True); q.add_argument("--attempt",type=int,required=True); q.add_argument("--worker-id",required=True); q.add_argument("--worker-ttl-seconds",type=int,default=900); q.add_argument("--checkpoint-file"); q.set_defaults(func=cmd_heartbeat)
    q=sub.add_parser("recover"); q.add_argument("--run",required=True); q.set_defaults(func=cmd_recover)
    q=sub.add_parser("reconcile-side-effect"); q.add_argument("--run",required=True); q.add_argument("--task-id",required=True); q.add_argument("--attempt",type=int,required=True); q.add_argument("--receipt-file",required=True); q.set_defaults(func=cmd_receipt)
    q=sub.add_parser("record-rate-limit"); q.add_argument("--run",required=True); q.add_argument("--provider",required=True); q.add_argument("--retry-after-seconds",type=int,required=True); q.set_defaults(func=cmd_rate)
    return p

def main():
    a=parser().parse_args()
    try: return int(a.func(a))
    except Exception as e: print(json.dumps({"error":type(e).__name__,"message":str(e)},indent=2),file=sys.stderr); return 1
if __name__=="__main__": raise SystemExit(main())
