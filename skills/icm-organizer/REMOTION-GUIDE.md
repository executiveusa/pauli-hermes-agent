# ICM Organizer + Remotion: AI-Driven Animation Workflows

Use the ICM Organizer skill to structure Remotion video projects into transparent, agent-orchestrated workspaces.

## Why ICM for Remotion?

Remotion + ICM = AI agents that can:
- ✅ Understand your Remotion project structure instantly
- ✅ Generate code in the right folders at the right time
- ✅ Track composition status by reading files
- ✅ Iterate on animations without context bloat
- ✅ Hand off work to humans at review gates

Instead of: "Regenerate the whole video with this change" (wasteful)  
You get: "Update the title composition, re-render" (surgical)

## The Pipeline Form for Remotion Projects

```
my-video/
├─ CLAUDE.md                          # Entry: "I am [project], render me with..."
├─ CONTEXT.md                         # Root contract
├─ stages/
│  ├─ 01_brief/
│  │  ├─ CONTEXT.md
│  │  ├─ references/brand.md, tone.md
│  │  └─ output/brief.md             # Human-reviewed project definition
│  │
│  ├─ 02_composition-structure/
│  │  ├─ CONTEXT.md
│  │  ├─ references/remotion-standards.md
│  │  └─ output/structure.md         # Component layout plan
│  │
│  ├─ 03_generate-components/
│  │  ├─ CONTEXT.md
│  │  ├─ references/component-registry.md, brand-colors.md
│  │  └─ output/src/components/     # Generated Remotion JSX components
│  │
│  ├─ 04_animation-logic/
│  │  ├─ CONTEXT.md
│  │  ├─ references/animation-principles.md, easing-guide.md
│  │  └─ output/src/sequences/      # Animation keyframes + timings
│  │
│  ├─ 05_render-config/
│  │  ├─ CONTEXT.md
│  │  ├─ references/render-specs.md, encoding-templates.md
│  │  └─ output/remotion.config.ts  # Render settings
│  │
│  └─ 06_render-and-deliver/
│     ├─ CONTEXT.md
│     ├─ references/delivery-specs.md
│     └─ output/final.mp4, final.mov
│
├─ src/
│  ├─ components/                    # Reusable Remotion components
│  ├─ sequences/                     # Animation sequences
│  ├─ Root.tsx                       # Main Remotion composition
│  └─ index.ts                       # Entry
│
├─ _shared/
│  ├─ brand-colors.md               # Color palette
│  ├─ animation-principles.md       # Easing, timing, style guide
│  ├─ component-registry.md         # Available components + props
│  ├─ remotion-standards.md         # Best practices, performance tips
│  └─ delivery-specs.md             # Output formats, codecs
│
└─ _templates/
   ├─ video-project-template/       # Copy for new videos
   └─ component-template.tsx         # Remotion component boilerplate
```

## Stage Breakdown

### 01_brief — Project Definition
**Input:** Project request (client spec, marketing brief, etc.)
**Process:**
1. Extract key messaging, style references, duration
2. Define target audience and platform
3. List required elements (text, graphics, animations, voiceover)

**Output:** `brief.md` (human reviews and approves)

### 02_composition-structure — Layout & Flow
**Input:** Approved brief
**Process:**
1. Plan scene structure (opening, middle, close)
2. Define composition hierarchy (containers, stacks, layers)
3. Specify timing (when each element appears/animates)

**Output:** `structure.md` (human reviews for pacing/flow)

### 03_generate-components — Write Remotion Code
**Input:** Approved structure + component registry
**Process:**
1. Generate JSX components matching the structure
2. Each component: props-driven, testable, reusable
3. Include comments for animation points
4. Reference design system (colors, fonts, spacing)

**Output:** `src/components/*` (human reviews code quality)

### 04_animation-logic — Easing, Timing, Motion
**Input:** Components + approved brief
**Process:**
1. Define keyframes and easing for each animated element
2. Synchronize animations with audio/voiceover if present
3. Add secondary motion (breathing, subtle loops)
4. Ensure performance (no expensive operations in frames)

**Output:** `src/sequences/*` (human reviews smoothness/timing)

### 05_render-config — Output Settings
**Input:** Approved animations
**Process:**
1. Set resolution, frame rate, codec
2. Configure audio mixing if needed
3. Define output file format and quality
4. Set parallelization for faster renders

**Output:** `remotion.config.ts` (human reviews specs)

### 06_render-and-deliver — Export Final Video
**Input:** Render config + all components/sequences
**Process:**
1. Render video using `remotion render` with config
2. Verify output against delivery specs
3. Generate backup formats if needed (MP4, MOV, etc.)
4. Create delivery package

**Output:** `final.mp4`, `final.mov` (human downloads/deploys)

## Using the Skill

### Create a Remotion Project Workspace

```bash
/icm-organizer build
```

**Describe:**
```
"I'm building Remotion videos. Process:
1. Client gives me a brief
2. I plan the composition structure  
3. I generate Remotion components
4. I add animations and timing
5. I configure render settings
6. I render and deliver MP4"
```

The skill scaffolds:
- Pipeline workspace matching your sequence
- Stage folders for each step
- `_shared/` with Remotion best practices and component registry
- Templates for new videos

### Integrate with Hermes

Hermes can now:

```bash
# Hermes reads the structure
ls stages/02_composition-structure/output/
# → structure.md exists, ready for next stage

# Hermes generates code in the right folder
cd stages/03_generate-components/output/src/components/
# → Write new component here

# Hermes tracks status
find stages/*/output/ -name "*.md" -o -name "*.tsx"
# → Status derivable from what exists
```

## The Key Insight

**Instead of:** "Here's a 50k-token monolith about my Remotion project, regenerate the whole thing"

**You get:** "Read the brief, write a component, save to `03_generate-components/output/`. I'll review, then you add animations."

Each stage loads 2k–8k tokens (brief + structure + component registry), not the entire project. Agents stay focused. Humans review at natural gates. Videos ship faster.

## Reference Material

Stage contracts point to these `_shared/` files:

- **brand-colors.md** — Hex values, Pantone references, usage rules
- **animation-principles.md** — Easing curves (ease-in-out, cubic-bezier), timing conventions, performance dos/don'ts
- **component-registry.md** — Available components, their props, example usage
- **remotion-standards.md** — Frame rate, codec settings, Remotion antipatterns
- **delivery-specs.md** — Client requirements (resolution, format, framerate, file size limits)

These are stable across every video project. Stage contracts reference them instead of restating them.

## Template: New Video Project

Copy `_templates/video-project-template/` when starting a new video:

```bash
cp -r _templates/video-project-template/ ../new-client-video
cd ../new-client-video
# Fill in CLAUDE.md with project name
# Add brief to 01_brief/output/brief.md
# Start with stage 02
```

Every new video has the same structure. Agents understand it instantly.

## Advanced: Multiple Videos in One Workspace

Use the **Umbrella form** if you're managing multiple Remotion projects with shared branding:

```
production-studio/
├─ CLAUDE.md (the map: which project is where)
├─ 01_brand-standards/        (shared factory)
├─ 02_component-library/      (shared components)
├─ 03_video-project-a/        (full Pipeline)
├─ 04_video-project-b/        (full Pipeline)
└─ 05_video-project-c/        (full Pipeline)
```

Each project is self-contained but shares brand/components.

## Performance Tips

1. **Keep stage inputs lean** — don't load all previous outputs, only what's needed
2. **Components should be pure** — props in, JSX out, no side effects
3. **Animations should be frame-driven** — use `interpolate()` for smooth motion
4. **Render in stages** — split long videos into sequences, compose later
5. **Cache reference materials** — `_shared/` files change rarely, load once

## Example: A 60-Second Explainer Video

**Timeline:**
- Stage 1 (brief): 30 min — define messaging
- Stage 2 (structure): 1 hour — plan scenes
- Stage 3 (components): 2 hours — generate base JSX
- Stage 4 (animation): 1.5 hours — add motion
- Stage 5 (render config): 30 min — set output
- Stage 6 (render): 15 min — export video

**Total:** ~5 hours, agent-assisted, human-reviewed at each gate.

---

**Your Hermes agent now understands Remotion workflows as structured files.** No more context bloat. Just numbered folders, plain text contracts, and a clear path from brief to delivery.

🎬 Let's make videos smarter!
