# Cosmos live-persona load boundary — Phase 1 evidence

## Verified in source code (not proof of a running instance)

`agent/prompt_builder.py:load_soul_md()` resolves `get_hermes_home() / "SOUL.md"` and `agent/system_prompt.py` uses it as the identity slot when present. `hermes_cli/default_soul.py` seeds a default identity on first run. This means **changing repository-root SOUL.md does not, by itself, prove that a running Hermes will adopt that persona**.

This repository also contains `agents/personas/cosmos/SOUL.md` and a literal historical `~/.hermes/SOUL.md` path in the source tree. Neither proves which file the actual Surface process used.

## Still unverified

The Surface was OFFLINE in Desktop Commander during this change. There is no current process environment, active profile, `HERMES_HOME` resolution, file hash, startup log, or authenticated runtime behavior receipt. Therefore **the running instance's exact persona file cannot be reported as verified**.

## Future live read-only verification (before applying any files)

1. Confirm Surface connection and executable/process identity and command line **without printing credentials**.
2. Resolve its actual `HERMES_HOME` and loaded `SOUL.md` path in the running process, and record a content hash (not private persona text in logs).
3. Compare with repository-root and other persona copies; inspect startup behavior and determine whether `HEART.md` and `CONTEXT.md` are actually loaded.
4. Before any authorized application, create timestamped backup copies of **live** persona files and confirm restore instructions and unchanged uncommitted checkout.
5. **WAIT for Bambu's separate explicit approval** before copying, reloading or restarting anything. Never pull or reset the Surface checkout.

## Rollback

This Phase 1 PR only changes GitHub branch files and is reversible by closing/reverting it. It makes **no live persona modification**. Applying the content to an active installation requires a separate owner approval and independent proof.
