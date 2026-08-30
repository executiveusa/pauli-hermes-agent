# ICM Context Contract

## Job
Stable factory/reference contracts Hermes must not guess.

## Inputs
Only context files explicitly named by the active instruction.

## Outputs
None at runtime. Runtime evidence belongs in `../memory/` or the active product record.

## Rules
- one authoritative home per contract;
- model/provider names stay behind routing;
- no secrets;
- private/client facts stay scoped;
- missing boundary contract = stop and report, never guess.

## Human check
Changes to access, autonomy, identity, external-system contracts or financial limits require owner review.
