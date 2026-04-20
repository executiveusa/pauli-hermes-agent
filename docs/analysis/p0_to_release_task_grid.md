# P0 → Release Task Grid

| Workstream | Current State | Planned Deliverables | Exit Criteria |
|---|---|---|---|
| A Discovery | Completed in this PR | Inventory + risk + env/workflow docs | Artifacts updated and versioned |
| B Upgrade Hygiene | Blocked by upstream fetch policy | Upstream merge branch + conflict notes | Upstream sync verified + green tests |
| C GitHub Operator | Partial (skills/auth only) | Dedicated adapter, indexing workers, workflow trigger tools, audit log | End-to-end GitHub repo ops tested |
| D Vercel Operator | Partial (provider alias only) | Vercel adapter, deployment diagnostics, redeploy actions, dashboard card | Deployment issue triage tested |
| E Infisical | Missing | bootstrap + machine auth + secret sync + health checks | Secret plane active without leaking values |
| F Twilio Voice | Missing (SMS exists) | voice inbound/outbound + STT/TTS + session transcript persistence | successful phone conversation loop |
| G Dashboard UX | Existing dashboard base | operator widgets (repo/deploy/secret/voice health) | usability pass + docs |
| H Knowledge Plane | Partial (memory plugins) | ingestion/indexing/search/attribution pipeline | reproducible ingest+query with sources |
| I Hostinger/Coolify | Partial (Dockerfile only) | compose/coolify config + runbooks + healthchecks | successful deployment rehearsal |
| J CI/CD | Existing workflows | PR gate docs + operator tests + security/lint/type/build parity | required checks green |
