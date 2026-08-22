---
name: social-drop-factory
description: Build and operate portable Social Drops: a weekly content system that turns Monday/Wednesday/Friday social posts into one coherent interactive micro-experience, with GitHub as source of truth and Webflow as an optional visual design lab. Use for recurring social campaigns, personalized sales pages, recruiting campaigns, nonprofit/community campaigns, local-service lead generation, product launches, coaching/mentorship content, and client-specific interactive microsites.
---

# Social Drop Factory

Use this skill when a business needs **consistent weekly content that feels designed, tells one coherent story, and gives attention somewhere useful to go**.

The canonical implementation lives at:

- `executiveusa/social-media-template`

## What this is

A **Social Drop** is a small, portable interactive campaign experience built from normal web primitives.

The core pattern is:

```text
MONDAY         WEDNESDAY       FRIDAY
BELIEF   ->    STORY      ->   ACTION
static post    Reel/story      static post
      \           |           /
       \          |          /
        -> INTERACTIVE DROP -> CTA
```

The social post is the cover. The Drop is the deeper experience.

The first proven prototype uses:

- native HTML/CSS
- native `<details>/<summary>` progressive-disclosure interaction
- no required Webflow runtime
- no required custom JavaScript for the core interaction
- responsive desktop/tablet/mobile layouts
- one shared renderer with client/campaign content supplied separately

## Source-of-truth architecture

Treat these roles as fixed unless the owner explicitly changes them:

```text
GitHub                    = canonical source of truth
Webflow                   = visual R&D / client preview / design laboratory
Static HTML/CSS           = portable production baseline
Client manifests + assets = campaign-specific data
Analytics / CRM           = separate measurement layer
Social scheduler          = separate distribution layer
```

Do **not** make Webflow the only copy of a campaign. Do **not** fork the renderer for each customer.

A new customer should be a **new manifest + assets folder**, not a new application.

## Canonical weekly grammar

For standard recurring social campaigns:

- **Monday — BELIEF / IDENTITY / EDUCATION**
  - establishes the idea, value, problem, principle, or category
- **Wednesday — STORY / HUMAN PROOF**
  - Reel, short video, case moment, mentor/customer/employee story, demonstration
- **Friday — ACTION / COMMUNITY / CONVERSION**
  - event, offer, booking, application, donation, purchase, participation, next step

Publishing order is:

```text
Monday -> Wednesday -> Friday
```

For an Instagram three-column row, the finished display order becomes:

```text
Friday | Wednesday | Monday
```

This is intentional. Preserve it when grid consistency is part of the client's system.

## Core workflow

### 1. Identify the weekly governing idea

Write one sentence that all three pieces serve.

Example:

> Mentorship turns values into visible community action.

If the three posts do not reinforce one idea, the week is fragmented. Fix the story before designing assets.

### 2. Compile the week

Translate the governing idea into:

```text
Monday    = what should the audience understand or believe?
Wednesday = what human story or proof makes it real?
Friday    = what should the audience do next?
```

### 3. Build the Drop

Use 3-7 cards/steps maximum unless evidence justifies more.

Useful primitives:

- Cover
- Stack
- Reveal
- Story
- Compare
- Choose
- Timeline
- Video
- Proof
- CTA

A Drop should usually end with **one dominant next action**.

### 4. Render from the shared engine

Prefer the repository's shared HTML/CSS renderer.

Do not create client-specific code when a manifest field, asset, token, or reusable component can express the variation.

### 5. Preview visually

Use Webflow when visual experimentation, client review, art-direction exploration, or builder-native editing is valuable.

Webflow is optional for production. The portable version must remain usable outside Webflow.

### 6. Run parity and quality gates

At minimum verify:

- desktop layout
- tablet layout
- mobile layout
- Monday/Wednesday/Friday content roles
- final Friday | Wednesday | Monday grid when applicable
- asset correctness
- CTA correctness
- no horizontal overflow
- interaction works without custom dependencies
- source-of-truth repo contains the current campaign definition

If porting from Webflow, structural/CSS parity is not the same as pixel parity. Do not claim pixel-perfect parity until screenshots are compared at defined viewport sizes.

### 7. Measure the business outcome

Track events such as:

```text
open
card_2_reached
card_3_reached
video_started
video_completed
cta_reached
cta_clicked
lead_created
application_started
purchase_or_booking
```

Do not optimize for posting volume alone. Optimize for the selected business outcome.

## High-value use cases

This system is not limited to social-media management.

Use it for:

1. **Recurring brand content** — weekly belief -> story -> action campaigns.
2. **Personalized sales** — one prospect-specific Drop containing a short video, observations, proof, offer, and booking CTA.
3. **Recruiting** — culture/belief -> employee story -> role -> application.
4. **Local-service lead generation** — educate around a problem -> show a real job/result -> offer estimate/inspection/booking.
5. **Nonprofit / community campaigns** — mission -> human/program story -> donate/volunteer/attend.
6. **Coaches, mentors, sports programs** — principle -> athlete/mentor story -> exercise/event/application.
7. **Product / ecommerce launches** — insight/problem -> demo -> use-case/proof -> buy.
8. **Motivational / creator content** — principle -> personal story -> challenge -> subscribe/join.
9. **Event campaigns** — why the event matters -> behind-the-scenes/human proof -> RSVP/attend.
10. **Customer onboarding / handoff** — promise -> walkthrough -> next action, replacing a dead PDF with an interactive client-specific experience.

See `references/use-cases.md` for patterns and manifest ideas.

## Personalized sales pattern

This is a priority use case.

```text
HEY <PROSPECT>
    ↓
30-60 second personal video
    ↓
WHAT I FOUND
    ↓
OBSERVATION 1
OBSERVATION 2
OBSERVATION 3
    ↓
WHAT I WOULD CHANGE
    ↓
PROOF / BEFORE-AFTER / EXAMPLE
    ↓
ONE OFFER
    ↓
BOOK / REPLY / BUY
```

The page should feel created for one buyer, even though the rendering engine is shared.

Example conceptual manifest:

```json
{
  "type": "personalized-sales",
  "prospect": "Nate",
  "company": "Acme Roofing",
  "headline": "I found three things costing your site leads",
  "observations": ["...", "...", "..."],
  "proof": "...",
  "offer": "Roofing Lead Rescue",
  "cta": "Book 15 minutes"
}
```

## Recruiting pattern

```text
WHAT WE BELIEVE
      ↓
WHAT THE WORK ACTUALLY LOOKS LIKE
      ↓
EMPLOYEE / TEAM STORY
      ↓
THE ROLE
      ↓
WHY IT MATTERS
      ↓
APPLY
```

Avoid generic "we're hiring" copy. Show the work before showing the job post.

## Design law

Do not produce generic AI social slop.

The factory automates **structure and repetition**, not taste.

Every campaign should have:

- a governing idea
- a distinctive visual system
- coherent typography and spacing
- one dominant CTA
- a reason for every motion or interaction
- progressive disclosure rather than information dumping
- clear mobile hierarchy
- proof before claims

Use independent review when stakes justify it. The builder should not be the final approver of its own visual work.

## Scaling law

The scalable unit is:

```text
new client = configuration + content + assets
```

Not:

```text
new client = new codebase
```

Target architecture:

```text
social-media-template/
├── engine/
│   ├── renderer
│   ├── components
│   └── shared styles
├── clients/
│   ├── client-a/
│   │   ├── brand.json
│   │   └── campaigns/
│   └── client-b/
├── dist/
└── tests/
```

See `references/architecture.md` for the operating model.

## Current prototype truth

The source repo currently documents:

- structural parity implemented
- source/DOM/CSS parity implemented from Webflow extraction
- visual screenshot parity not yet certified
- browser behavior parity not yet certified

Preserve that distinction. Never upgrade a parity claim without evidence.

## When to activate this skill

Strong triggers include:

- "social drop"
- "social media template"
- "weekly content system"
- "Monday Wednesday Friday content"
- "build this week's campaign"
- "personalized sales page"
- "personalized outreach microsite"
- "make a recruiting drop"
- "turn this post into an interactive experience"
- "make a campaign from these three posts"
- "use the social-media-template repo"
- "make this scalable across clients"

## Required output for major work

For material campaign work, report:

```text
DECISION
CHANGES
PROOF
STATUS
COMMERCIAL IMPACT
RISKS
ROLLBACK
NEXT
HUMAN APPROVAL (when publishing or other consequential action is involved)
```

Never claim a production publish, analytics result, lead result, visual parity pass, or campaign performance result without evidence.