# Dependency Map (Snapshot: 2026-04-23)

## Package Managers and Lockfiles
- Python: `pyproject.toml`, `requirements.txt`, `uv.lock`.
- Node.js (root): `package.json`, `package-lock.json`.
- Node.js (subprojects): `ui-tui/package.json`, `web/package.json`, `website/package.json`, `scripts/whatsapp-bridge/package.json`.
- Nix: `flake.nix`, `flake.lock`.

## Runtime Surfaces
- Kernel orchestration: `run_agent.py`, `model_tools.py`, `toolsets.py`.
- Tooling layer: `tools/registry.py` + `tools/*.py`.
- CLI: `cli.py` + `hermes_cli/*`.
- Gateway/messaging: `gateway/*`.
- TUI stack: `ui-tui/*` + `tui_gateway/*`.
- Dashboard/API: `hermes_cli/web_server.py` + `web/*`.

## Integration Capability Snapshot
- GitHub: present in auth/model/skills and CI workflows; no first-class dedicated GitHub operator module under `tools/` yet.
- Vercel: deploy hook used in `.github/workflows/deploy-site.yml`; no first-class Vercel deployment operator module under `tools/` yet.
- Twilio: SMS platform adapter exists (`gateway/platforms/sms.py`); no first-class voice operator pipeline detected.
- Infisical: no dedicated adapter/service module detected in current code tree.

## Operational Conclusion
The repository is a strong Hermes kernel with many extension points already in place, but P0 operator capabilities (GitHub repo operations, Vercel deployment diagnosis/redeploy, Infisical secret plane, Twilio voice) still need dedicated implementation modules and tests.
