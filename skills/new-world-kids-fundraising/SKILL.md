---
name: new-world-kids-fundraising
description: Operate New World Kids fundraising and grants through FundRazr, HSI fiscal-sponsor governance, evidence-backed grant drafting, compliant browser/computer use, donor traffic, and reporting. Use for NW Kids fundraising, FundRazr, HSI, grant research, grant writing, donor campaigns, fundraising traffic, or grant pipeline work.
---

# New World Kids Fundraising

Route the task through `hermes-workflows/new-world-kids-fundraising/AGENTS.md`.

## Operating rules

1. Load the workflow route and only the stage context needed for the current task.
2. FundRazr is the primary HSI-provided digital fundraising platform. Prefer native/authorized integrations before browser automation.
3. Never persist sponsor/member credentials. Use local secret storage/environment variables only.
4. Never automate GrantStation research unless written permission from GrantStation explicitly allows the workflow. A human may research there and hand the opportunity record to Hermes.
5. Every grant must receive HSI staff review before submission, regardless of amount. Owner approval remains required for final submission and financial/public-claim gates.
6. Draft from evidence. Unsupported claims stay marked `NEEDS SOURCE`.
7. For traffic, use canonical campaign URLs + UTMs, approved media, and human-approved outbound sequences.
8. For reporting, distinguish gross/net, submitted/awarded, pledged/received, and verified/unverified.

## Browser/computer use

Use browser control to assist supervised FundRazr setup, verify public pages, inspect donor flows, and work with public sources whose terms permit it. Do not bypass access controls or silently change payout, tax/legal, campaign-launch, or financial settings.

## Completion

Return the current stage, artifact written, proof used, blocker/approval required, and the single next action. Do not call the workflow complete until required human/HSI gates and source checks are satisfied.
