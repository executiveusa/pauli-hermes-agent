# Story Archetype Router — LOCKED

Every content run must record:

```yaml
story:
  primary_story_type: ""
  secondary_story_type: null
  classification_source: user_selected | inferred | asked
  confidence: high | medium | low
```

If confidence is low, ask. Do not guess.

## Canonical types

### 1. Overcame It
Adversity → struggle → earned outcome.
Use when sustained struggle is the spine.
Do not use when a single decision or surprise is the true hinge.
Question: **Is this mainly about someone overcoming a real obstacle over time?**

### 2. Nobody Saw This Coming
Surprising cause/effect.
Use when the unexpected turn is the reason to keep watching.
Do not manufacture surprise.
Question: **Is the main value that the cause or result is genuinely unexpected?**

### 3. The Person Behind It
Turn an organization, business, or project into a human story.
Use when a person makes the organization understandable.
Question: **Is the point to humanize a company, project, or organization through the person behind it?**

### 4. Before It Was Fixed
Problem → intervention → proof.
Use when there is a real before-state, intervention, and after-state.
Question: **Can we show the problem, what changed, and proof of the result?**

### 5. One Decision Changed Everything
Decision-centered story.
Use when one choice is the hinge between before and after.
Question: **Is there one specific decision that clearly changed what happened next?**

### 6. Why This Matters
Issue made concrete through one person/event.
Use when a human example makes a larger issue understandable.
Question: **Are we using one real person or event to make a bigger issue feel concrete?**

### 7. Receipts
Claim → evidence → result.
Use when trust and proof are the story.
Question: **Is the strongest version of this content proving a claim with evidence?**

### 8. Challenge
Call out → tension → challenge/action.
Use when the destination is a specific viewer action.
Question: **Are we directly challenging the viewer to do or change something?**

### 9. Hidden Opportunity
Something valuable being overlooked.
Use when discovery itself creates the value.
Question: **Is the core value revealing something useful that most people are overlooking?**

### 10. Interactive Choice
Audience chooses the next reveal/branch.
Use only when the viewer's choice genuinely changes what they see next.
Status: **EXPERIMENT SHELF**.
Interactive Sora or another branching-video provider may support this later, but it is NOT a core publishing dependency.

## Classification protocol

### Rule 1 — explicit user choice wins
If the user picks a type and the source supports it, use it. If the source contradicts it, state the mismatch briefly and offer the nearest valid choices.

### Rule 2 — infer when obvious
Examples:

- “She lost everything, rebuilt, and opened the store.” → `Overcame It`
- “Prove response time improved from two hours to three minutes.” → `Receipts`
- “Tell the founder story behind this nonprofit.” → `The Person Behind It`

### Rule 3 — ask only when needed
First ask:

**What is the main thing the viewer should feel, understand, or do?**

Offer only the 3–5 most likely choices from context, not all ten unless necessary.

If still ambiguous, ask:

**What is the strongest evidence or turning point we actually have?**

Stop once confidence is high.

## Separation rules

### Overcame It vs One Decision Changed Everything
- sustained struggle → `Overcame It`
- one specific choice → `One Decision Changed Everything`

### Nobody Saw This Coming vs Hidden Opportunity
- surprise about cause/result drives retention → `Nobody Saw This Coming`
- overlooked usable value drives the story → `Hidden Opportunity`

### Before It Was Fixed vs Receipts
- transformation process is the story → `Before It Was Fixed`
- proving the claim is the story → `Receipts`

### The Person Behind It vs Why This Matters
- person humanizes an organization/project → `The Person Behind It`
- person/event humanizes a larger issue → `Why This Matters`

### Challenge vs Why This Matters
- destination is viewer action → `Challenge`
- destination is understanding/caring → `Why This Matters`

### Interactive Choice vs all others
`Interactive Choice` is primary only when branching itself is central. Branches may contain other archetypes as branch-level metadata. Never require Interactive Sora unless an experiment explicitly enables it.

## Pre-script gate

Before scripting, establish:

```yaml
content_run:
  primary_story_type: ""
  secondary_story_type: null
  reason: ""
  source_evidence: []
  engine:
    hook: ""
    tension: ""
    payoff: ""
    action: ""
```

If `primary_story_type` is missing: **STOP**.

## Long-form and Shorts

The same router applies to both.

For Shorts, keep one clear story spine. Avoid combining multiple archetypes in a short runtime unless the secondary is clearly subordinate.

For long-form, one archetype remains the episode spine; chapters may use secondary archetypes without changing the episode promise.

## Learning compatibility

Story type is a required analytics dimension. Compare like-with-like before changing creative doctrine.
