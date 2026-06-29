# Blockers

## Confirmed External Blockers

1. Provider credential/runtime mismatch
   - Hermes API health passes, but actual chat completions return downstream `401 Missing Authentication header`
   - OpenAI direct key from the local secret corpus also validated as invalid

2. Hermes Desktop dependency/install integrity on this Windows host
   - `npm install` / `npm ci --ignore-scripts` never completed into a stable state
   - `.bin` wrappers were not generated and key package trees were incomplete for their expected launch files

3. Deployment access completeness
   - `COOLIFY_BASE_URL` missing
   - Hostinger deploy credentials/access not found

4. Recommended Linux path unavailable
   - WSL2 kernel exists, but no distro is installed
   - Docker is not installed

5. OpenChronicle runtime on this host
   - Upstream `OpenChronicle` currently documents itself as macOS-only
   - It can be designed into the Pauli browser/memory layer, but it cannot be truthfully activated as always-on local context capture on this Windows machine today

## Non-Blocking Observations

- Dirty root repo was preserved intact
- Clean Hermes worktree and desktop clone were created/used safely
- Hermes local API listener itself is healthy on `127.0.0.1:8642`
