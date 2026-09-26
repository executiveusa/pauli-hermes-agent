---
name: find-skills
description: Helps users discover and install agent skills when they ask questions like "how do I do X", "find a skill for X", "is there a skill that can...", or express interest in extending capabilities. This skill should be used when the user is looking for functionality that might exist as an installable skill.
metadata:
  author: vercel
  version: "1.0.0"
  source: https://github.com/vercel-labs/skills
---

# Find Skills

This skill helps you discover and install skills from the open agent skills ecosystem.

## When to Use This Skill

Use this skill when the user:

- Asks "how do I do X" where X might be a common task with an existing skill
- Says "find a skill for X" or "is there a skill for X"
- Asks "can you do X" where X is a specialized capability
- Expresses interest in extending agent capabilities
- Wants to search for tools, templates, or workflows
- Mentions they wish they had help with a specific domain (design, testing, deployment, etc.)

## Two Ways to Search: `hermes skills` (native, preferred) vs `npx skills`

This agent ships with its own skill hub, so **prefer `hermes skills` first** — it runs in-process
(no Node/npx dependency) and crawls the same catalog (skills.sh, ClawHub, GitHub, LobeHub, the
Claude marketplace):

```bash
hermes skills search <query>          # search the index
hermes skills browse                  # interactive browse
hermes skills inspect <identifier>    # preview a skill's SKILL.md before installing
hermes skills install <identifier>    # install it into skills/
hermes skills list                    # see what's already installed
```

Fall back to the [Vercel `npx skills` CLI](https://github.com/vercel-labs/skills) when you need
something `hermes skills` doesn't cover — e.g. installing a skill for a *different* coding agent
(Cursor, Copilot, OpenCode, etc.), or resolving an unusual source format (a direct GitHub tree
URL, a `.zip`/`.tar.gz` archive, a GitLab URL):

```bash
npx skills find [query] [--owner <owner>]   # search
npx skills add <owner/repo>                  # install
npx skills add <owner/repo>@<skill> -a claude-code -g -y   # install one skill, globally, non-interactive
npx skills update                            # update installed skills
```

**Browse skills at:** https://skills.sh/

## How to Help Users Find Skills

### Step 1: Understand What They Need

When a user asks for help with something, identify:

1. The domain (e.g., React, testing, design, deployment)
2. The specific task (e.g., writing tests, creating animations, reviewing PRs)
3. Whether this is a common enough task that a skill likely exists

### Step 2: Check What's Already Bundled

Before searching externally, check whether this repo already ships a skill for the domain —
browse `skills/` (categories like `agent-skills`, `mlops`, `productivity`, `creative`, `devops`,
etc.) or grep for the keyword. No need to install anything if it's already there.

### Step 3: Search for Skills

If nothing bundled covers it, search the ecosystem:

```bash
hermes skills search <query>
# or, for the broader multi-agent catalog:
npx skills find [query] [--owner <owner>]
```

For example:

- User asks "how do I make my React app faster?" → `hermes skills search react performance`
- User asks "can you help me with PR reviews?" → `hermes skills search pr review`
- User asks "I need to create a changelog" → `hermes skills search changelog`

Well-known high-quality sources worth checking directly: `vercel-labs/agent-skills` (React,
Next.js, web design — already partly vendored in this repo under `skills/agent-skills/`) and
`anthropics/skills` (frontend design, document processing).

### Step 4: Verify Quality Before Recommending

**Do not recommend a skill based solely on search results.** Always verify:

1. **Install count** — Prefer skills with 1K+ installs. Be cautious with anything under 100.
2. **Source reputation** — Official sources (`vercel-labs`, `anthropics`, `microsoft`) are more trustworthy than unknown authors.
3. **GitHub stars** — Check the source repository. A skill from a repo with <100 stars should be treated with skepticism.

### Step 5: Present Options to the User

When you find relevant skills, present them to the user with:

1. The skill name and what it does
2. The install count and source
3. The install command they can run
4. A link to learn more at skills.sh

Example response:

```
I found a skill that might help! The "react-best-practices" skill provides
React and Next.js performance optimization guidelines from Vercel Engineering.
(185K installs)

To install it:
hermes skills install vercel-labs/agent-skills@react-best-practices

Learn more: https://skills.sh/vercel-labs/agent-skills/react-best-practices
```

### Step 6: Offer to Install

If the user wants to proceed, install it for them with the native command:

```bash
hermes skills install <owner/repo@skill>
```

Or, if it targets a different agent or source type `hermes skills` can't resolve:

```bash
npx skills add <owner/repo@skill> -g -y
```

## Common Skill Categories

When searching, consider these common categories:

| Category        | Example Queries                          |
| --------------- | ---------------------------------------- |
| Web Development | react, nextjs, typescript, css, tailwind |
| Testing         | testing, jest, playwright, e2e           |
| DevOps          | deploy, docker, kubernetes, ci-cd        |
| Documentation   | docs, readme, changelog, api-docs        |
| Code Quality    | review, lint, refactor, best-practices   |
| Design          | ui, ux, design-system, accessibility     |
| Productivity    | workflow, automation, git                |

## Tips for Effective Searches

1. **Use specific keywords**: "react testing" is better than just "testing"
2. **Try alternative terms**: If "deploy" doesn't work, try "deployment" or "ci-cd"
3. **Check popular sources**: Many skills come from `vercel-labs/agent-skills` or `ComposioHQ/awesome-claude-skills`

## When No Skills Are Found

If no relevant skills exist:

1. Acknowledge that no existing skill was found
2. Offer to help with the task directly using your general capabilities
3. Suggest the user could create their own skill — `hermes skills` doesn't have an `init`
   scaffolding command, so write the `SKILL.md` by hand (see any existing skill under `skills/`
   for the expected format) or use `npx skills init my-xyz-skill` if the npx CLI is available.

Example:

```
I searched for skills related to "xyz" but didn't find any matches.
I can still help you with this task directly! Would you like me to proceed?

If this is something you do often, I can write you a SKILL.md for it.
```
