# Portfolio Connection Contract

Hermes is the operating layer for the GitHub portfolio. Repositories remain sovereign codebases; Hermes indexes and routes work without copying client intelligence into every repo.

## Source of truth
Use the authenticated `executiveusa` GitHub App installation at runtime to enumerate accessible repositories and resolve current default branch, visibility, permissions, recent commits, open PRs, CI state, and README/AGENTS guidance before making changes.

## Connection rules
1. Inspect before changing.
2. Never assume the cached portfolio list is complete; enumerate the live installation.
3. Do not bulk-edit every repo to install Client OS files.
4. Store cross-project operating memory and client intelligence in Hermes.
5. Keep repo-specific code, deployment config, tests, and ownership records inside that repo.
6. Before modifying any repo, record baseline SHA, branch, checks, blast radius, rollback, and proof target.
7. After modifications, re-read README/AGENTS guidance and update only when the change materially affects them.
8. Builders cannot self-approve production claims; production status requires external evidence such as deployment health, runtime checks, or client-visible verification.

## Portfolio classification
Every repo should eventually be tagged SELL, USE, MERGE, PARK, or ARCHIVE. Only SELL repos should enter active revenue work without replacing an existing active workstream.

## Active workstream cap
- one revenue offer
- one shared platform
- one bounded experiment

Everything else is parked until explicitly promoted.
