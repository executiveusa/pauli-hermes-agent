# Cosmos autonomous runner — phase 3: fail closed

**Owner decision (2026-11-19):** Hermes/Cosmos is the spec-and-CodeDen-handoff architect, not a coding factory, mission queue, deployment orchestrator, or pipeline runner.

The legacy `autonomous_runner.py` previously constructed an `AIAgent` with `deploy_apps` and marked repositories `deploying` based on a model-produced JSON boolean. A model decision is NOT authenticated owner approval. That direct path has now been replaced with a compatibility shim that **always raises `OwnerGateRequired` before any model/database/network/production action**. This cannot be enabled by environment variables or unverified approval files. No production changes or standing schedules are started by this PR.

**This is deliberately more restrictive than merely an off-by-default flag.** The external owner-release policy gate's exact API and authentication mechanism have not been verified. A future separately approved and independently reviewed change must implement action-specific owner authorization through the installed policy boundary, including exact repo, commit, operation, budget, expiry and rollback, *without reinstating a parallel Hermes coding factory*. Until that happens this entrypoint is entirely disabled, even with a purported approval.

Regression tests: `python -m unittest tests/test_cosmos_autonomous_runner_gate.py -v`.

Rollback: revert the PR in GitHub if compatibility restoration is explicitly owner-approved; restoring the prior autonomous deploy behavior is **not safe without an actual policy gate**. Never run this on an active Surface checkout without backing up its persona files, preserving uncommitted work and receiving Bambu's separate approval.
