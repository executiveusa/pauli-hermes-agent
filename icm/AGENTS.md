# Hermes ICM Router

Read `CONTEXT.md` first.

- Any consequential repository work → `instructions/REPO_WALK_TEST.md` before implementation.
- Cross-repo/product dependency work → `context/PROJECT_REGISTRY.md` plus only directly material repo state.
- Revenue / prospect / offer / growth work → `instructions/HERMES.md` then `instructions/PROOF_FIRST_REVENUE_LOOP.md`.
- Capability/model requirements → `context/TASK_PROFILES.md`.
- Worker/judge message shape → `context/ENVELOPES.md`.
- Sequential proof-first execution → `workflows/proof-first-revenue-loop/CONTEXT.md`.
- Run evidence/history → `memory/CONTEXT.md`.

Runtime code stays outside `icm/`. ICM is the operating catalog and contract layer; load only what the current step names.

Identity boundary: the owner's personal Hermes is distinct from MACS Digital Media's Agent Max/runtime. Never share client memory, credentials, runtime state, or authority by convenience.