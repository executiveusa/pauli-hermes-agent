# Hermes Phase 01 Baseline Report

## DECISION

Do not wholesale-merge upstream Hermes. The fork is deeply diverged and already contains a large portion of the modern upstream execution substrate. Reconciliation must be selective and implementation-aware.

## CHANGES

Documentation only:

- added OpenSpec-style phase proposal
- recorded exact fork/upstream relationship
- corrected the capability inventory based on current source
- defined Phase 02 comparison policy

## PROOF

### Repository truth

- Pauli fork: `executiveusa/pauli-hermes-agent`
- fork flag: `true`
- parent/source: `NousResearch/hermes-agent`
- Pauli baseline SHA: `950aabef00059cbbf8a8a735da0bc215acd2d483`
- upstream head at audit: `4aa9f738cedbe8a69fbd08595d0fb67f812ce2d3`
- merge base: `c79e3fd0baf41c0adda616b73153eeaa8a4b8231`
- comparison state: `diverged`
- ahead: 163 commits
- behind: 11,864 commits

### Runtime substrate already present

Current `toolsets.py` includes:

- `terminal`, `process`
- `read_file`, `write_file`, `patch`, `search_files`
- browser navigate/snapshot/click/type/scroll/back/press/images/vision/console/CDP/dialog
- `skills_list`, `skill_view`, `skill_manage`
- `todo`, `memory`, `session_search`
- `execute_code`, `delegate_task`
- `cronjob`
- `send_message`
- Kanban show/list/complete/block/heartbeat/comment/create/link/unblock
- `computer_use`
- `vps_ssh_execute`, `ralphy_run_task`, `ralphy_prd_sync`

`pyproject.toml` identifies Hermes Agent version `0.16.0`, Python `>=3.11,<3.14`, exact-pinned core dependencies, an MCP extra, a computer-use extra, web/dashboard support, messaging integrations, and development test tooling.

### Computer use correction

Current Pauli Hermes already has a registered macOS `computer_use` toolset backed by `cua-driver` and MCP. The Windows gap remains real: a Windows desktop backend still needs separate inspection/design rather than pretending the whole computer-use capability is absent.

### CI/deployment baseline

GitHub combined status on the earlier `3c3f9a60...` baseline showed two Vercel status failures whose target was Vercel's account-deployment-blocked knowledge page. No GitHub Actions workflow runs were returned for that commit. Deployment work remains outside this coding sprint until the intended Vercel team/account is restored and verified.

## STATUS

PHASE 01 IMPLEMENTATION: COMPLETE

Merge status: NOT YET MERGED. Phase PR/review gate still required.

## RISKS

1. Very large upstream drift means a blanket sync would have extreme conflict/regression risk.
2. Existing Pauli capability may be mistaken for missing upstream capability if we compare documentation instead of source.
3. Repo does not currently have `EMERALD_TABLETS.md` or an `openspec/` tree on main; this phase introduces the first scoped OpenSpec change path without rewriting existing governance.
4. Local sandbox checkout could not be completed because the coding sandbox cannot resolve `github.com`; code phases must use GitHub-side CI or a materialized checkout when available before claiming test proof.

## ROLLBACK

Revert this phase's documentation commit(s). No runtime or data rollback is required.

## NEXT

Phase 02: build the implementation-level upstream capability matrix and identify the smallest safe first port/hardening slice.
