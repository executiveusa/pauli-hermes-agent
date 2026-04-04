---
name: skill-creation-law
description: Mandatory template and rules for creating new skills
platforms: [cli, telegram, discord, slack, api]
---

# Skill Creation Law

## Purpose
Ensures all new skills follow a consistent structure and meet quality standards.

## Rules

1. **Naming**: All lowercase, hyphens for spaces (e.g. `api-migration`). No underscores, no spaces.
2. **Structure**: Every skill MUST have YAML frontmatter with `name`, `description`, `platforms`.
3. **Sections**: Every SKILL.md MUST contain: Purpose, Rules (numbered), Checklist (checkbox format).
4. **Rules**: Minimum 3 rules, maximum 10. Each rule is one clear sentence.
5. **Checklist**: Minimum 3 items. Each item is verifiable (pass/fail, not subjective).
6. **Platform scope**: Specify which platforms the skill applies to. Default: all.
7. **No duplication**: Before creating, check `skill_list_all` for existing skills on the topic.
8. **Testing**: After creation, verify the skill loads with `skill_view`.
9. **Maintenance**: Skills MUST be updated when they're found to be wrong or outdated.

## Template
```markdown
---
name: <skill-name>
description: <one-sentence description>
platforms: [cli, telegram, discord, slack, api]
---

# <Skill Name>

## Purpose
<What this skill governs and why>

## Rules
1. <Rule one>
2. <Rule two>
3. <Rule three>

## Checklist
- [ ] <Verifiable item>
- [ ] <Verifiable item>
- [ ] <Verifiable item>
```

## Checklist
- [ ] Name follows lowercase-hyphen convention
- [ ] YAML frontmatter complete (name, description, platforms)
- [ ] Purpose section present and clear
- [ ] 3-10 numbered rules present
- [ ] 3+ checklist items present
- [ ] No duplicate of existing skill
- [ ] Skill loads successfully with skill_view
