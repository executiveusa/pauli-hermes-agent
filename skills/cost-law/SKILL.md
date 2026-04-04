---
name: cost-law
description: Token and infrastructure cost management
platforms: [cli, telegram, discord, slack, api]
---

# Cost Law

## Purpose
Manages token usage and infrastructure costs to stay within budget.

## Rules

1. **Model selection**: Use the cheapest model that can do the job well.
   - Trivial tasks (greetings, simple lookups): Mercury 2 / Gemini Flash
   - Standard tasks (coding, analysis): Claude Sonnet / GPT-4o
   - Complex tasks (architecture, long reasoning): Claude Opus / o1
2. **Token discipline**: Don't send unnecessary context. Compress when over 50% of context window.
3. **Caching**: Never break prompt caching mid-conversation. Static content first, dynamic content last.
4. **Batch operations**: Group multiple small operations into single tool calls when possible.
5. **Monthly budget**: Target $30/month total across all agents and infrastructure.
6. **Cost tracking**: Log model usage and costs. Review weekly.
7. **Infrastructure**: Use self-hosted (VPS) over managed services where the quality is equivalent.

## Model Routing Table

| Task Type | Model | Estimated Cost |
|-----------|-------|----------------|
| Simple chat | mercury-coder-small | ~$0.001/msg |
| Web search synthesis | gemini-flash | ~$0.002/msg |
| Code generation | claude-sonnet | ~$0.01/msg |
| Complex reasoning | claude-opus | ~$0.05/msg |
| Vision analysis | gemini-flash | ~$0.003/img |

## Checklist
- [ ] Cheapest viable model selected for task
- [ ] Prompt caching not broken
- [ ] Context compressed if over 50%
- [ ] Monthly spend under budget
