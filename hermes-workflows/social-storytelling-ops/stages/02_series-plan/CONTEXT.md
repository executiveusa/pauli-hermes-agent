# 02 Series Plan
Map the story graph to the campaign outcome and publishing cadence. Define each Reel's role, viewer belief before/after, emotional function, CTA role, dependencies on static posts/events, and sequence. Output `series-plan.md` + machine-readable manifest. Reject isolated 'viral clips' that do not advance the series.

## Weekly cadence

Slot incoming story material into `skills/social-drop-factory`'s
Monday/Wednesday/Friday belief → story → action cadence rather than
publishing reels as disconnected one-offs. This stage does not duplicate
that cadence logic — it references it:

- Read `skills/social-drop-factory/SKILL.md` §"Core workflow" steps 1–2
  before assigning any Reel's role. Step 1 (identify the weekly governing
  idea — one sentence all three pieces serve) comes first; a Reel's
  before/after viewer belief in this stage's output is that governing
  idea's before/after, not an independent invention per Reel.
- Map `story-map.json` candidates to cadence slots:
  - **Monday (BELIEF)** — a Reel whose function is establishing the idea,
    value, or problem the series is about. Emotional function: understand.
  - **Wednesday (STORY)** — a Reel carrying the human proof/demonstration
    from the source transcript. Emotional function: feel it's real.
  - **Friday (ACTION)** — a Reel whose CTA role matches the week's single
    dominant next action (`social-drop-factory` §"Build the Drop": "a Drop
    should usually end with one dominant next action").
- A story graph with source material for only one or two of the three
  roles is a partial week — flag it rather than force-filling the missing
  role with unrelated or invented content. `story-miner`'s "no invented
  copy" rule extends here.
- Multiple weeks form a series only when consecutive weeks' governing
  ideas are dependent (this stage's existing `dependencies on static
  posts/events` field) — do not silently start a new unrelated governing
  idea every week; that recreates the isolated-clip problem this stage
  already rejects, one level up.
- If the client's campaign also runs an interactive Drop (not just three
  social posts), `campaign-architect` treats the Drop as the Friday
  destination per `social-drop-factory`'s source-of-truth architecture
  (GitHub canonical, Webflow optional preview) — this stage does not own
  Drop rendering, only the reference from the Friday Reel's CTA.