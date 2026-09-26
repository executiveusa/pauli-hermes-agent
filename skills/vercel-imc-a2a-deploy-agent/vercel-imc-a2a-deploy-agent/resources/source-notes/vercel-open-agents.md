# Vercel Open Agents Notes

Design decisions imported into this workspace:

- Treat Open Agents as a forkable reference architecture, not a black box.
- Preserve separation between web/control workflow and sandbox execution.
- Keep the agent outside the execution sandbox and interact with it using file, shell, git, and browser tools.
- Use Vercel durable/runtime primitives only when the workspace moves from local folder kit to hosted service.
