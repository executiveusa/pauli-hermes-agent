# HyperAgent routines

This directory stores verified HyperAgent action-cache routines learned by Hermes/Cosmos.

Rules:

- one routine per JSON file;
- stable descriptive names only;
- no secrets, passwords, cookies, tokens, payment data, or private form values;
- preserve the previous verified version until a repaired replacement passes proof;
- a routine is reusable only after target identity and final-state verification;
- Agent Max must not consume these routines during the Bambu/Cosmos pilot.

Runtime-generated routine JSON files should be reviewed before commit. Most routine caches should remain local operational state rather than source-controlled artifacts unless deliberately promoted as a safe template.
