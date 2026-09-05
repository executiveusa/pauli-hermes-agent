# PARÉ release status — 2026-09-04

- Canonical repo: `executiveusa/PARE`.
- Owner approved release merge with exact word `approve`.
- PR #5 merged to `main` as merge commit `c0e65919a54e2a7331d33ce256a2ea2960be6b6b`.
- Merged PR head: `44bc3038985b7b9d846c113be317c276222d4e85`.
- The previous hero defect was architectural: the page used a generated random-letter matrix with four target glyphs, not a real crossword.
- PR #5 now contains a real static crossword structure anchored by PARÉ plus scroll-driven subtraction/convergence.
- Post-merge hero verification continues in PARÉ PR #6 from branch `fix/pare-crossword-hero-postmerge-20260904`; it adds regression coverage so the hero cannot silently return to the old random-letter matrix.
- Netlify production must not be called current for this release until its deployed commit is proven to equal the approved main revision (or a documented derivative build of it).
- VPS/runtime gates remain separate from hero correctness: daemon, public API, real providers, Studio SSE, diffusion, persistence, MCP/API/CLI, fresh verifier, Gauntlet.

## Personal Hermes instruction
Treat PARÉ `main` as the source of truth for release code. Before consequential PARÉ work, run the repository walk test and compare `main`, the active bounded PR, Netlify production SHA, and VPS SHA. Never merge personal-Hermes authority into MACS Digital Media / Agent Max.