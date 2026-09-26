---
name: tool-use-budget-policy
description: Budget policy for choosing tools and limiting wasteful agent execution.
---

# Tool Use Budget Policy

- Prefer the least expensive tool that can complete the task.
- Avoid duplicate calls when one call is enough.
- Stop and replan if the task grows beyond the current budget.
- Explain cost-sensitive decisions in the run log.
