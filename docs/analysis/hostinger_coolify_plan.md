# Hostinger + Coolify Plan

## Current Deploy Readiness

- `COOLIFY_API_KEY`: present somewhere in scanned env corpus
- `COOLIFY_BASE_URL`: missing
- Hostinger API/SSH credential variables requested by the mission were not found
- Docker is not installed locally
- WSL distro is not installed locally

## Result

Local documentation and script scaffolding could be prepared, but actual Coolify discovery or staging deployment cannot proceed safely without:

1. A real Coolify base URL
2. Hostinger staging/VPS access details
3. A container/runtime path for parity testing (Docker or a configured WSL distro)
