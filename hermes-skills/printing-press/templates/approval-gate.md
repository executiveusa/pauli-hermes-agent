# Approval Gate Template

## Command
`<slug>-pp-cli <command>`

## Risk class
`read_only|write|financial|legal`

## Gate policy
- `read_only`: autonomous execution allowed.
- `write`: explicit user approval required.
- `financial`: explicit user approval + two-person review.
- `legal`: explicit user approval + two-person review.

## Preflight checks
- Auth scope minimal for requested action.
- Target environment/account explicitly identified.
- Dry-run or preview mode used when supported.

## Approval record
- Requestor:
- Approver:
- Timestamp (UTC):
- Evidence link:
