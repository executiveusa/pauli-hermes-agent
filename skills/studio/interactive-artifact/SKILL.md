---
name: interactive-artifact
description: Studio skill — Interactive Artifact.
version: 1.0.0
author: Bambú / Pauli Effect
---

# Interactive Artifact Skill

Loaded for autonomous studio agents. Full source below.

## When to Use
Use when the task matches this skill's domain.

## Source
`C:\Users\execu\Downloads\AI WORKSHOP\MASTER SKILLS BUNDLE\agent-must-read-this\interactive-artifact-skill.md`

---

---
name: interactive-artifacts
description: >
  Builds production-quality interactive HTML artifacts that render inline in Claude.ai and
  any MCP-capable agent environment. Covers the full build stack: CSS variable theming,
  dark-mode-safe color system, Tabler icon integration, Chart.js and D3 data visualization,
  three.js 3D scenes, Anthropic API-powered AI widgets, persistent storage, sendPrompt()
  agent handoff, and the claude.ai iframe sandbox constraints. Use this skill when any agent
  is asked to build, improve, or audit an interactive artifact — dashboards, scorers,
  simulators, games, calculators, form mockups, agent graph explorers, data visualizers,
  cost guards, or any widget that must run live in the chat UI. Enforces Emerald Tablets™
  quality floor: UDEC 8.5/10, anti-hype law, single-responsibility, Ralphy Loop output.
  Blocks all known sandbox failure modes before code is written.
emerald_tablets: I, II, IV, V
quality_floor: 8.5
author: Pauli Effect™ × Akash Engine
version: "1.0"
---

# Interactive Artifacts — Agent Build Skill™
## Pauli Ecosystem™ Constitutional Skill
### Authority: Emerald Tablets™ I · II · IV · V
### Applies to: HERMES™ · RALPHY · ARCHITECT · any agent producing claude.ai widgets

---

## PRIME DIRECTIVE

Read this file completely before writing a single line of HTML, CSS, or JavaScript.

An interactive artifact is not a webpage. It is a sandboxed iframe rendered inside claude.ai.
It has no `<html>`, no `<head>`, no `<body>`. It cannot access `localStorage`. It cannot reach
arbitrary external domains. It cannot use `position: fixed`. Every failure mode in this document
is a real failure that has silently broken artifacts before. Knowing the constraints is how you
build correctly on the first iteration.

The quality floor is 8.5/10. Anything below it auto-iterates. Anything that silently fails
a sandbox constraint counts as a 0 on the affected axis.

---

## STEP 0 — MANDATORY CONTEXT SCAN

Before writing any artifact code, run this check:

```
1. What is the output environment?
   → Always claude.ai iframe sandbox unless explicitly told otherwise.
   → Constraints in SECTION 2 apply unconditionally.

2. What data or state does this artifact need?
   → Static data: embed in JS variables.
   → User-session data: use in-memory JS state (never localStorage).
   → Cross-session persistence: use window.storage API (SECTION 7).
   → Real-time AI responses: use Anthropic API pattern (SECTION 8).

3. Does the artifact need external libraries?
   → Only load from the CDN allowlist in SECTION 3.
   → Check exact UMD build URLs — wrong paths silently 404.

4. What is the sendPrompt() handoff strategy?
   → Every meaningful user action should offer an agent follow-up.
   → See SECTION 9 for sendPrompt() patterns.

5. What is the UDEC score target per axis?
   → Run self-audit in SECTION 10 before declaring done.
```

---

## SECTION 1 — ARTIFACT ANATOMY

Every artifact is a raw HTML fragment. No boilerplate wrappers.

### What to NEVER include
```html
<!-- BANNED — will break or be ignored -->
<!DOCTYPE html>
<html>
<head>
<body>
<!-- HTML comments -->
/* CSS comments */
```

### Correct document structure
```html
<style>
  /* styles first — stream before content */
  /* keep under ~20 lines for simple artifacts */
  /* complex interactive widgets may need more — that is fine */
</style>

<!-- content HTML second -->
<div>...</div>

<!-- scripts last — execute after streaming completes -->
<script src="https://cdnjs.cloudflare.com/..."></script>
<script>
  /* your code here */
</script>
```

### Streaming order matters
The artifact streams token-by-token. Structure so useful content appears early:
- `<style>` block → renders CSS before DOM
- Content HTML → user sees layout immediately
- `<script src="...">` CDN loads → libraries available
- `<script>` logic → executes after all above

---

## SECTION 2 — SANDBOX CONSTRAINTS (HARD LIMITS)

These are not preferences. Violating any of these produces broken artifacts.

### CONSTRAINT 1: No localStorage or sessionStorage
```javascript
// BANNED — silently fails in claude.ai sandbox
localStorage.setItem('key', value);
sessionStorage.getItem('key');

// CORRECT — in-memory state
let appState = { user: null, data: [] };

// CORRECT — cross-session persistence
await window.storage.set('key', value); // see SECTION 7
```

### CONSTRAINT 2: No position: fixed
```css
/* BANNED — collapses iframe to 100px height */
.modal { position: fixed; top: 0; left: 0; }
.toast { position: fixed; bottom: 20px; }

/* CORRECT — use normal flow for overlays */
.modal-wrap {
  min-height: 400px;
  display: flex;
  align-items: center;
  justify-content: center;
}
```

### CONSTRAINT 3: CDN allowlist only
External resources load ONLY from these domains:
```
cdnjs.cloudflare.com
esm.sh
cdn.jsdelivr.net
unpkg.com
fonts.googleapis.com
fonts.gstatic.com
```
Any other origin is silently blocked. No GitHub raw URLs. No custom CDNs.

### CONSTRAINT 4: No DOCTYPE, html, head, body tags
The sandbox injects these. Duplicate tags break rendering.

### CONSTRAINT 5: Canvas cannot resolve CSS variables
```javascript
// BANNED — canvas ignores CSS vars, renders transparent/black
ctx.fillStyle = 'var(--text-primary)';

// CORRECT — read CSS vars into JS first
const style = getComputedStyle(document.documentElement);
const textColor = style.getPropertyValue('--text-primary').trim();
ctx.fillStyle = textColor;
```

### CONSTRAINT 6: No nested scrolling
Do not set `overflow: scroll` or `overflow: auto` on inner containers.
The iframe auto-sizes to content height. Let it.

### CONSTRAINT 7: No gradients, shadows, blur, glow during streaming
These flash during DOM diffing. Use solid fills. Exception: after-stream
CSS animations using `transform` and `opacity` only are permitted.

---

## SECTION 3 — CDN LIBRARY REFERENCE

### Verified UMD build URLs (copy exactly — wrong paths 404 silently)

```html
<!-- Chart.js 4.4.1 — data visualization -->
<script src="https://cdnjs.cloudflare.com/ajax/libs/Chart.js/4.4.1/chart.umd.js"></script>

<!-- D3 7.8.5 — advanced visualization, choropleth maps -->
<script src="https://cdnjs.cloudflare.com/ajax/libs/d3/7.8.5/d3.min.js"></script>

<!-- TopoJSON 3.0.2 — geographic maps (requires D3) -->
<script src="https://cdnjs.cloudflare.com/ajax/libs/topojson/3.0.2/topojson.min.js"></script>

<!-- Three.js r128 — 3D scenes -->
<script src="https://cdnjs.cloudflare.com/ajax/libs/three.js/r128/three.min.js"></script>

<!-- PapaParse — CSV parsing -->
<script src="https://cdnjs.cloudflare.com/ajax/libs/PapaParse/5.4.1/papaparse.min.js"></script>

<!-- SheetJS — Excel XLSX/XLS parsing -->
<script src="https://cdn.jsdelivr.net/npm/xlsx@0.18.5/dist/xlsx.full.min.js"></script>

<!-- mathjs — mathematical expressions -->
<script src="https://cdnjs.cloudflare.com/ajax/libs/mathjs/12.4.0/math.min.js"></script>

<!-- Mermaid 11 — diagrams (ES module only) -->
<script type="module">
  import mermaid from 'https://esm.sh/mermaid@11/dist/mermaid.esm.min.mjs';
</script>

<!-- Tabler icons — already loaded in claude.ai, NO import needed -->
<!-- Use directly: <i class="ti ti-home"></i> -->
```

### Three.js r128 specific constraints
```javascript
// AVAILABLE in r128
THREE.OrbitControls  // NOT available — use manual rotation
THREE.BoxGeometry
THREE.SphereGeometry
THREE.CylinderGeometry
THREE.MeshStandardMaterial

// NOT AVAILABLE in r128 (added in later versions)
THREE.CapsuleGeometry  // use CylinderGeometry + SphereGeometry instead
THREE.OrbitControls    // implement manual mouse drag instead
```

---

## SECTION 4 — COLOR SYSTEM

### CSS variables — always use these, never hardcode hex in HTML/CSS
```css
/* Surfaces (light/dark auto-adapts) */
--surface-0   /* page background — darkest */
--surface-1   /* card background */
--surface-2   /* panel — white in light, elevated dark */
--surface-3   /* popover */

/* Text */
--text-primary      /* body text */
--text-secondary    /* supporting text */
--text-muted        /* placeholders, hints */

/* Semantic roles */
--text-accent       --bg-accent     --border-accent
--text-danger       --bg-danger     --border-danger
--text-success      --bg-success    --border-success
--text-warning      --bg-warning    --border-warning
--text-pro          --bg-pro        --border-pro

/* Borders */
--border            /* default 0.5px hairline */
--border-strong     /* emphasized */
--border-stronger   /* heavy */

/* Typography */
--font-sans   /* Anthropic Sans — default */
--font-voice  /* serif — editorial moments only */
--font-mono   /* monospace */

/* Layout */
--radius      /* 8px — standard corners */
--pad-sm --pad-md --pad-lg --pad-xl
--gap-xs --gap-sm --gap-md --gap-lg --gap-xl
```

### Nine-ramp color palette for data visualization
Use these hex values in Chart.js and D3 (canvas cannot read CSS vars):

```javascript
const PALETTE = {
  // Tidepool categorical series — always assign in this order
  series: ['#2a78d6','#1baf7a','#eda100','#008300','#4a3aa7','#e34948','#e87ba4','#eb6834'],

  // Named ramps for UI components (50=lightest, 900=darkest)
  purple: { 50:'#EEEDFE', 100:'#CECBF6', 200:'#AFA9EC', 400:'#7F77DD', 600:'#534AB7', 800:'#3C3489', 900:'#26215C' },
  teal:   { 50:'#E1F5EE', 100:'#9FE1CB', 200:'#5DCAA5', 400:'#1D9E75', 600:'#0F6E56', 800:'#085041', 900:'#04342C' },
  coral:  { 50:'#FAECE7', 100:'#F5C4B3', 200:'#F0997B', 400:'#D85A30', 600:'#993C1D', 800:'#712B13', 900:'#4A1B0C' },
  amber:  { 50:'#FAEEDA', 100:'#FAC775', 200:'#EF9F27', 400:'#BA7517', 600:'#854F0B', 800:'#633806', 900:'#412402' },
  blue:   { 50:'#E6F1FB', 100:'#B5D4F4', 200:'#85B7EB', 400:'#378ADD', 600:'#185FA5', 800:'#0C447C', 900:'#042C53' },
  green:  { 50:'#EAF3DE', 100:'#C0DD97', 200:'#97C459', 400:'#639922', 600:'#3B6D11', 800:'#27500A', 900:'#173404' },
  red:    { 50:'#FCEBEB', 100:'#F7C1C1', 200:'#F09595', 400:'#E24B4A', 600:'#A32D2D', 800:'#791F1F', 900:'#501313' },
  gray:   { 50:'#F1EFE8', 100:'#D3D1C7', 200:'#B4B2A9', 400:'#888780', 600:'#5F5E5A', 800:'#444441', 900:'#2C2C2A' },
  pink:   { 50:'#FBEAF0', 100:'#F4C0D1', 200:'#ED93B1', 400:'#D4537E', 600:'#993556', 800:'#72243E', 900:'#4B1528' },
};

// Dark mode detection for charts
const isDark = matchMedia('(prefers-color-scheme: dark)').matches;
```

### Color assignment rules
```
Categorical data     → Tidepool series array in order — never pick randomly
Sequential data      → one hue, light→dark (e.g. blue 50→900)
Diverging data       → blue ↔ red with gray midpoint (#f0efec light / #383835 dark)
Status indicators    → semantic CSS vars (--text-success, --text-danger, etc.)
Never               → rainbow cycling, hardcoded hex in CSS, color as only differentiator
```

---

## SECTION 5 — TYPOGRAPHY AND COMPONENT RULES

### Type scale
```css
/* Headings — sentence case always, never ALL CAPS or Title Case */
h1 { font-size: 22px; font-weight: 500; }
h2 { font-size: 18px; font-weight: 500; }
h3 { font-size: 16px; font-weight: 500; }

/* Body */
p  { font-size: 16px; font-weight: 400; line-height: 1.7; }

/* UI labels */
.label  { font-size: 14px; }
.caption { font-size: 13px; }
.micro  { font-size: 12px; }
/* Minimum: 11px. Never go below. */
```

### Two weights only
```css
font-weight: 400; /* regular */
font-weight: 500; /* medium/bold */
/* Never 600, 700, 800 — too heavy against claude.ai chrome */
```

### Pre-styled form elements — write bare tags, no custom CSS needed
```html
<!-- These are already styled by the host: -->
<input type="text" placeholder="Enter value">
<input type="range" min="0" max="100" value="50">
<select><option>Option A</option></select>
<textarea rows="3"></textarea>
<button>Action label</button>
```

### Button convention
```html
<!-- Standard button — uses pre-styled outline variant -->
<button onclick="handleClick()">Do something</button>

<!-- Agent handoff button — append ↗ arrow -->
<button onclick="sendPrompt('Tell me more about X')">Deep dive on X ↗</button>
```

### Card anatomy
```html
<!-- Raised card -->
<div style="background:var(--surface-2);border-radius:12px;border:0.5px solid var(--border);padding:1rem 1.25rem;">
  content
</div>

<!-- Metric card (for KPI numbers) -->
<div style="background:var(--surface-1);border-radius:var(--radius);padding:1rem;">
  <div style="font-size:11px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.5px;margin-bottom:4px;">Label</div>
  <div style="font-size:22px;font-weight:500;font-family:var(--font-voice);">$42,000</div>
</div>
```

### Badge/pill anatomy
```html
<!-- Status badge — text must use same-ramp dark color, never plain black -->
<span style="background:var(--bg-success);color:var(--text-success);border:0.5px solid var(--border-success);font-size:12px;padding:2px 8px;border-radius:4px;">Active</span>

<!-- Category tag using color ramp -->
<span style="background:#EEEDFE;color:#3C3489;border:0.5px solid #534AB7;font-size:11px;padding:2px 8px;border-radius:4px;">Agent</span>
```

### Tabler icons — already loaded, use immediately
```html
<!-- Inline with text -->
<i class="ti ti-check" aria-hidden="true"></i> Done

<!-- Icon-only button — must have aria-label -->
<button aria-label="Delete item"><i class="ti ti-trash"></i></button>

<!-- Sizing: 16-20px inline, 24px max decorative -->
<i class="ti ti-settings" style="font-size:20px"></i>

<!-- Common icons available: -->
<!-- ti-home ti-settings ti-user ti-search ti-x ti-check ti-plus ti-trash -->
<!-- ti-edit ti-download ti-upload ti-file ti-folder ti-chart-bar ti-calendar -->
<!-- ti-clock ti-arrow-right ti-arrow-left ti-chevron-down ti-external-link -->
<!-- ti-copy ti-refresh ti-player-play ti-player-pause ti-heart ti-star -->
<!-- ti-bell ti-mail ti-lock ti-eye ti-menu-2 ti-alert-triangle ti-ban -->
<!-- NEVER use -filled suffix (ti-heart-filled) — not loaded, renders blank -->
```

---

## SECTION 6 — LAYOUT PATTERNS

### Container width
The widget container is 680px wide. Design for this width.

```css
/* Responsive grid — auto-fit handles column count */
.grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(160px, 1fr)); gap: 12px; }

/* Grid with clamped columns — prevents overflow */
.grid-3 { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 10px; }
/* Use minmax(0, 1fr) not 1fr — prevents child content pushing columns past container */
```

### Spacing system
```css
/* Vertical rhythm — use rem */
margin-top: 1rem;    /* 16px */
margin-top: 1.5rem;  /* 24px */
margin-top: 2rem;    /* 32px */

/* Component-internal gaps — use px */
gap: 8px;
gap: 12px;
gap: 16px;
```

### Full-width section pattern
```html
<div style="padding: 1.5rem 0; border-bottom: 0.5px solid var(--border);">
  section content
</div>
```

### Stepper / multi-step pattern
```html
<!-- Progress indicator -->
<div style="display:flex;gap:8px;margin-bottom:1.5rem;">
  <span style="width:8px;height:8px;border-radius:50%;background:var(--text-primary)"></span>
  <span style="width:8px;height:8px;border-radius:50%;background:var(--border-strong)"></span>
  <span style="width:8px;height:8px;border-radius:50%;background:var(--border-strong)"></span>
</div>

<!-- Step container — JS swaps content, never display:none during stream -->
<div id="step-content" style="min-height:200px;">
  <!-- filled by JS after stream completes -->
</div>
<div style="display:flex;gap:8px;margin-top:1rem;">
  <button onclick="prev()">← Back</button>
  <button onclick="next()">Next →</button>
</div>
```

### Avoid these layout traps
```
❌ display: none on any element during initial render — hidden content streams invisibly
❌ tabs / carousels during streaming — same issue
❌ overflow: auto on inner divs — creates nested scroll
❌ position: fixed anywhere — collapses iframe
❌ table-layout default with many columns — overflows 680px; use table-layout: fixed
```

---

## SECTION 7 — PERSISTENT STORAGE API

Use `window.storage` for cross-session data. This is the ONLY persistence mechanism
available. No localStorage. No cookies. No IndexedDB.

### API surface
```javascript
// Store value (personal — only this user sees it)
await window.storage.set('key', JSON.stringify(data));

// Store value (shared — all users of this artifact see it)
await window.storage.set('key', JSON.stringify(data), true);

// Retrieve value
const result = await window.storage.get('key');
const data = result ? JSON.parse(result.value) : null;

// Delete value
await window.storage.delete('key');

// List keys with prefix
const { keys } = await window.storage.list('prefix:');
```

### Key naming rules
```
✓ 'todos:todo-001'          hierarchical, no spaces
✓ 'user-prefs'              simple kebab-case
✓ 'scores:2026-06-29'       date-suffixed
✗ 'my key'                  spaces banned
✗ 'data/records'            slashes banned
✗ 'user\'s data'            quotes banned
```

### Error handling pattern (non-existent keys throw, not return null)
```javascript
async function loadData(key, fallback = null) {
  try {
    const result = await window.storage.get(key);
    return result ? JSON.parse(result.value) : fallback;
  } catch {
    return fallback;
  }
}

async function saveData(key, data) {
  try {
    await window.storage.set(key, JSON.stringify(data));
    return true;
  } catch (err) {
    console.error('Storage error:', err);
    return false;
  }
}
```

### Loading state pattern (show progressively, never block entire UI)
```javascript
async function init() {
  // Show skeleton immediately
  document.getElementById('content').innerHTML = '<p style="color:var(--text-muted)">Loading…</p>';

  const data = await loadData('app-state', defaultState);
  renderContent(data);
}
```

### Shared data disclosure
When an artifact uses `shared: true` storage, inform users with a note:
```html
<p style="font-size:12px;color:var(--text-muted);margin-top:8px;">
  <i class="ti ti-users" aria-hidden="true"></i>
  Data on this board is visible to all users of this artifact.
</p>
```

---

## SECTION 8 — ANTHROPIC API PATTERN (AI-POWERED ARTIFACTS)

Use the Anthropic API to build artifacts that call Claude internally.

### Endpoint and model
```javascript
const API_ENDPOINT = 'https://api.anthropic.com/v1/messages';
const MODEL = 'claude-sonnet-4-6'; // always this model, always
const MAX_TOKENS = 1000;           // always 1000 — handled by proxy
```

### Base call pattern
```javascript
async function callClaude(userMessage, systemPrompt = '') {
  const body = {
    model: MODEL,
    max_tokens: MAX_TOKENS,
    messages: [{ role: 'user', content: userMessage }],
  };
  if (systemPrompt) body.system = systemPrompt;

  const response = await fetch(API_ENDPOINT, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });

  const data = await response.json();
  return data.content.map(b => b.type === 'text' ? b.text : '').join('');
}
```

### JSON-only response pattern (structured output)
```javascript
async function callClaudeJSON(prompt) {
  const systemPrompt = `
    You are a data extraction agent.
    Respond ONLY with a valid JSON object.
    No preamble. No markdown. No backticks. Just JSON.
  `;

  const raw = await callClaude(prompt, systemPrompt);

  try {
    // Strip any accidental markdown fences
    const clean = raw.replace(/```json|```/g, '').trim();
    return JSON.parse(clean);
  } catch {
    console.error('JSON parse failed:', raw);
    return null;
  }
}
```

### Multi-turn conversation pattern
```javascript
// Maintain history in memory — API has no memory between calls
const conversationHistory = [];

async function chat(userMessage) {
  conversationHistory.push({ role: 'user', content: userMessage });

  const response = await fetch(API_ENDPOINT, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      model: MODEL,
      max_tokens: MAX_TOKENS,
      messages: conversationHistory,
    }),
  });

  const data = await response.json();
  const assistantText = data.content.find(b => b.type === 'text')?.text || '';
  conversationHistory.push({ role: 'assistant', content: assistantText });
  return assistantText;
}
```

### MCP server integration in API calls
```javascript
const body = {
  model: MODEL,
  max_tokens: MAX_TOKENS,
  messages: [{ role: 'user', content: userMessage }],
  mcp_servers: [
    { type: 'url', url: 'https://mcp.notion.com/mcp', name: 'notion-mcp' },
    { type: 'url', url: 'https://drivemcp.googleapis.com/mcp/v1', name: 'drive-mcp' },
  ],
};
```

### Parsing MCP tool results
```javascript
// MCP responses mix text blocks, tool_use blocks, and tool_result blocks
// Never assume position — always filter by type

const textBlocks = data.content.filter(b => b.type === 'text').map(b => b.text).join('\n');
const toolResults = data.content
  .filter(b => b.type === 'mcp_tool_result')
  .map(b => b.content?.[0]?.text || '')
  .join('\n');

// Parse tool result content as structured data, not string matching
try {
  const parsed = JSON.parse(toolResults);
  // use parsed.*
} catch {
  // use toolResults as text
}
```

### Loading state for API calls
```javascript
async function handleUserAction() {
  const btn = document.getElementById('action-btn');
  const output = document.getElementById('output');

  btn.disabled = true;
  btn.textContent = 'Thinking…';
  output.style.color = 'var(--text-muted)';
  output.textContent = 'Asking Claude…';

  try {
    const result = await callClaude(userInput);
    output.style.color = 'var(--text-primary)';
    output.textContent = result;
  } catch (err) {
    output.style.color = 'var(--text-danger)';
    output.textContent = 'Call failed. Check connection.';
  } finally {
    btn.disabled = false;
    btn.textContent = 'Ask Claude ↗';
  }
}
```

---

## SECTION 9 — sendPrompt() HANDOFF PATTERNS

`sendPrompt(text)` sends a message to the chat as if the user typed it.
Use it to pass control back to the main agent for tasks the artifact cannot handle.

### When to use sendPrompt()
```
✓ User wants to go deeper on a topic the artifact surfaced
✓ Artifact surface is too small for a full answer
✓ User action should trigger a new agent task
✓ Artifact result should become input to another workflow
✗ Simple state changes (toggle, filter, sort) — handle in JS instead
✗ Calculations — handle in JS instead
```

### Pattern examples
```html
<!-- Deep dive button -->
<button onclick="sendPrompt('Explain why my Feedback axis is scoring low in the UDEC scorer')">
  Why is this low? ↗
</button>

<!-- Context-aware handoff -->
<button onclick="sendPrompt(`Generate a SKILL.md for the ${currentAgentName} agent with scope: ${scopeText}`)">
  Generate skill file ↗
</button>

<!-- Dynamic prompt from artifact state -->
<script>
function requestImprovement(axis, score) {
  const prompt = `My systems design score on the ${axis} axis is ${score}/10. What are the highest-leverage fixes?`;
  sendPrompt(prompt);
}
</script>
<button onclick="requestImprovement('Feedback', currentScores.FBK)">Get improvement plan ↗</button>
```

### Always append ↗ to sendPrompt buttons
Signals to users that this action leaves the artifact and opens a chat response.

---

## SECTION 10 — CHART.JS PATTERNS

### Standard setup (every chart needs this)
```html
<div style="position:relative;width:100%;height:300px;">
  <canvas id="myChart" role="img" aria-label="[describe what chart shows]">
    [Fallback text: same data in prose for screen readers]
  </canvas>
</div>
<script src="https://cdnjs.cloudflare.com/ajax/libs/Chart.js/4.4.1/chart.umd.js"></script>
<script>
const isDark = matchMedia('(prefers-color-scheme: dark)').matches;
const root = getComputedStyle(document.documentElement);
const textMuted = root.getPropertyValue('--text-muted').trim() || (isDark ? '#898781' : '#898781');
const borderColor = isDark ? '#2c2c2a' : '#e1e0d9';

new Chart(document.getElementById('myChart'), {
  type: 'bar',
  data: {
    labels: ['Q1','Q2','Q3','Q4'],
    datasets: [{
      label: 'Revenue',
      data: [42000, 58000, 51000, 73000],
      backgroundColor: '#2a78d6',
      borderRadius: 4,
    }]
  },
  options: {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: { display: false }, // always disable default legend
    },
    scales: {
      x: {
        grid: { color: borderColor },
        ticks: { color: textMuted, font: { size: 12 } },
      },
      y: {
        grid: { color: borderColor },
        ticks: {
          color: textMuted,
          font: { size: 12 },
          callback: v => '$' + (v/1000).toFixed(0) + 'K', // format numbers
        },
      },
    },
  },
});
</script>
```

### Custom legend (always use instead of Chart.js default)
```html
<div style="display:flex;flex-wrap:wrap;gap:16px;margin-bottom:8px;font-size:12px;color:var(--text-secondary);">
  <span style="display:flex;align-items:center;gap:4px;">
    <span style="width:10px;height:10px;border-radius:2px;background:#2a78d6;"></span>Revenue
  </span>
  <span style="display:flex;align-items:center;gap:4px;">
    <span style="width:10px;height:10px;border-radius:2px;background:#1baf7a;"></span>Cost
  </span>
</div>
```

### Number formatting rules
```javascript
// ALL displayed numbers must be rounded — float math leaks
Math.round(value)           // counts, scores
value.toFixed(2)            // currency, percentages
value.toLocaleString()      // large numbers with commas
(v < 0 ? '-$' : '$') + Math.abs(v).toFixed(0)  // negative currency: -$5K not $-5K

// Range sliders — always set step to prevent float output
<input type="range" min="0" max="100" step="1" value="50">
```

### Horizontal bar chart height fix
```javascript
// wrapper height must be: (num_bars * 40) + 80px minimum
const wrapperHeight = (data.length * 40) + 80;
document.getElementById('chart-wrap').style.height = wrapperHeight + 'px';
```

### Axis label visibility (autoSkip causes missing labels)
```javascript
scales: {
  x: {
    ticks: {
      autoSkip: false,  // force all labels visible for ≤12 categories
      maxRotation: 45,
    }
  }
}
```

---

## SECTION 11 — ACCESSIBILITY REQUIREMENTS

Every artifact must pass these checks before shipping.

### Screen reader landmark
```html
<!-- HTML artifacts: visually hidden heading -->
<h2 style="position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0,0,0,0);">
  [One-sentence description of what this artifact does]
</h2>
```

### Color is never the only differentiator
```
Charts: pair color with dash pattern, shape, or label
Status: pair color with icon + text label
Categories: pair color with pattern fill where necessary
```

### Interactive element requirements
```html
<!-- Icon-only buttons must have aria-label -->
<button aria-label="Close dialog"><i class="ti ti-x"></i></button>

<!-- Canvas charts must have role + aria-label + fallback text -->
<canvas role="img" aria-label="Bar chart showing Q1-Q4 revenue">
  Revenue: Q1 $42K, Q2 $58K, Q3 $51K, Q4 $73K
</canvas>

<!-- Form inputs must have associated labels -->
<label for="name-input">Your name</label>
<input id="name-input" type="text">
```

### Focus management
```css
/* Never remove focus outlines — the pre-styled elements handle this */
/* Only override for custom interactive elements */
.custom-card:focus-visible {
  outline: 2px solid var(--text-accent);
  outline-offset: 2px;
}
```

---

## SECTION 12 — SELF-AUDIT CHECKLIST (UDEC 8.5 GATE)

Run this before declaring any artifact complete. Every axis must score ≥ 7.0.
Overall weighted average must be ≥ 8.5.

```
SANDBOX COMPLIANCE (weight: 15%)
  [ ] No localStorage or sessionStorage
  [ ] No position: fixed
  [ ] All external resources from CDN allowlist only
  [ ] No DOCTYPE / html / head / body tags
  [ ] No HTML or CSS comments
  [ ] Canvas color reads go through getComputedStyle, not CSS vars

DARK MODE SAFETY (weight: 10%)
  [ ] All text uses CSS variable tokens — no hardcoded hex in CSS
  [ ] Colored backgrounds use same-ramp dark text (800/900 stop)
  [ ] Canvas/Chart.js colors use JS-resolved hex with isDark check
  [ ] No mix of hardcoded hex backgrounds with CSS var foregrounds

ACCESSIBILITY (weight: 10%)
  [ ] Screen reader landmark present (h2 sr-only or aria-label)
  [ ] All canvas elements have role="img" + aria-label + fallback text
  [ ] Icon-only buttons have aria-label
  [ ] Color not used as sole differentiator
  [ ] No font-size below 11px

TYPOGRAPHY DISCIPLINE (weight: 8%)
  [ ] Sentence case on all visible text — no ALL CAPS, no Title Case
  [ ] Font weights: 400 and 500 only
  [ ] No mid-sentence bolding in rendered text
  [ ] Two font families maximum

NUMBER FORMATTING (weight: 7%)
  [ ] All displayed numbers go through Math.round / toFixed / toLocaleString
  [ ] Negative currency formatted as -$5K not $-5K
  [ ] Range sliders have step attribute set

LAYOUT INTEGRITY (weight: 10%)
  [ ] No nested overflow/scroll containers
  [ ] Grid uses minmax(0, 1fr) not 1fr where children might overflow
  [ ] Tables with many columns use table-layout: fixed
  [ ] No tabs or display:none content during initial render

STREAMING STRUCTURE (weight: 8%)
  [ ] Style block comes first
  [ ] Content HTML second
  [ ] Script tags last
  [ ] No gradients/shadows that flash during streaming

INTERACTIVITY QUALITY (weight: 12%)
  [ ] All meaningful user actions have loading states
  [ ] Error states handled (API failures, storage errors, parse errors)
  [ ] sendPrompt() buttons for appropriate agent handoff
  [ ] Every sendPrompt button has ↗ suffix
  [ ] Disabled state handled (not just ignored)

DATA VISUALIZATION (weight: 10%)
  [ ] Custom HTML legend replaces Chart.js default
  [ ] Series colors assigned from Tidepool palette in order
  [ ] autoSkip: false for ≤12 category axes
  [ ] Horizontal bar chart wrapper height calculated from data length
  [ ] No dual y-axis charts

EMERALD TABLETS™ COMPLIANCE (weight: 10%)
  [ ] No banned words: seamless, robust, innovative, leverage, synergy, utilize
      revolutionize, transforming, elevating, comprehensive, cutting-edge
  [ ] All measurements are specific (not "fast" — "loads in 0.8s")
  [ ] Single responsibility: artifact does one job well
  [ ] Ralphy Loop complete: ops/reports JSON produced if agent context
  [ ] LATAM specificity applied if Synthia™ 3.0 context
```

### Scoring
```
Each axis: 0–10
Weighted average: sum(score × weight) / sum(weights)
Floor: 8.5 overall, 7.0 per axis
Below floor on any axis: fix that axis before shipping
Below 8.5 overall: iterate — do not ship
```

---

## SECTION 13 — COMPLETE ARTIFACT TEMPLATES

### Template A: Interactive scorer / calculator
```html
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  .wrap { padding: 1.5rem 0; font-family: var(--font-sans); }
  .row { display: flex; align-items: center; gap: 12px; padding: 10px 0; border-bottom: 0.5px solid var(--border); }
  .row-label { font-size: 14px; color: var(--text-secondary); flex: 1; }
  .row-val { font-size: 14px; font-weight: 500; width: 36px; text-align: right; }
  .score-display { background: var(--surface-1); border-radius: 12px; padding: 1.25rem; text-align: center; margin-bottom: 1.5rem; }
  .score-big { font-size: 48px; font-weight: 500; font-family: var(--font-voice); line-height: 1; }
  .score-bar-wrap { height: 6px; background: var(--surface-0); border-radius: 3px; margin: 10px 0 6px; overflow: hidden; }
  .score-bar { height: 100%; border-radius: 3px; transition: width 0.3s ease, background 0.3s ease; }
</style>

<h2 style="position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0,0,0,0);">
  Interactive systems scorer — adjust axis sliders to calculate your UDEC score
</h2>

<div class="wrap">
  <div class="score-display">
    <div class="score-big" id="score-num">8.5</div>
    <div style="font-size:13px;color:var(--text-secondary);margin-top:4px;">weighted score</div>
    <div class="score-bar-wrap">
      <div class="score-bar" id="score-bar" style="width:85%;background:#1baf7a"></div>
    </div>
    <div style="font-size:13px;font-weight:500;" id="verdict">Passes floor</div>
  </div>

  <div id="axis-rows"></div>
  <button onclick="sendPrompt('How do I improve my lowest-scoring axis?')" style="width:100%;margin-top:1rem;">
    Get improvement plan ↗
  </button>
</div>

<script>
const axes = [
  { k: 'STK', label: 'Stock integrity', w: 10, val: 9 },
  { k: 'FBK', label: 'Feedback loops', w: 12, val: 9 },
  { k: 'RSL', label: 'Resilience',      w: 10, val: 9 },
  { k: 'SEC', label: 'Secrets safety',  w: 4,  val: 10 },
];

function render() {
  document.getElementById('axis-rows').innerHTML = axes.map(a => `
    <div class="row">
      <span class="row-label">${a.label}</span>
      <input type="range" min="1" max="10" step="1" value="${a.val}"
        oninput="update('${a.k}',+this.value)" style="flex:1">
      <span class="row-val" id="v-${a.k}">${a.val}</span>
    </div>
  `).join('');
  recalc();
}

function update(k, v) {
  axes.find(a => a.k === k).val = v;
  document.getElementById('v-' + k).textContent = v;
  recalc();
}

function recalc() {
  const totalW = axes.reduce((s, a) => s + a.w, 0);
  const weighted = axes.reduce((s, a) => s + a.val * a.w, 0);
  const score = (weighted / totalW).toFixed(1);
  const pct = Math.round(score / 10 * 100);
  const color = score >= 8.5 ? '#1baf7a' : score >= 7 ? '#eda100' : '#e34948';

  document.getElementById('score-num').textContent = score;
  document.getElementById('score-num').style.color = color;
  document.getElementById('score-bar').style.width = pct + '%';
  document.getElementById('score-bar').style.background = color;
  document.getElementById('verdict').textContent =
    score >= 8.5 ? 'Passes floor' : score >= 7 ? 'Iterate before shipping' : 'Below floor — fix blockers';
  document.getElementById('verdict').style.color =
    score >= 8.5 ? 'var(--text-success)' : score >= 7 ? 'var(--text-warning)' : 'var(--text-danger)';
}

render();
</script>
```

### Template B: AI-powered chat widget
```html
<style>
  .chat-wrap { padding: 1.5rem 0; }
  .messages { min-height: 200px; max-height: 400px; overflow-y: auto; margin-bottom: 1rem; }
  .msg { padding: 10px 14px; border-radius: 12px; margin-bottom: 8px; font-size: 14px; line-height: 1.6; max-width: 85%; }
  .msg.user { background: var(--bg-accent); color: var(--text-accent); margin-left: auto; border-radius: 12px 12px 4px 12px; }
  .msg.assistant { background: var(--surface-1); color: var(--text-primary); border-radius: 12px 12px 12px 4px; }
  .input-row { display: flex; gap: 8px; }
  .input-row input { flex: 1; }
</style>

<h2 style="position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0,0,0,0);">
  AI chat interface powered by Claude
</h2>

<div class="chat-wrap">
  <div class="messages" id="messages">
    <div class="msg assistant">Ask me anything about the Pauli ecosystem.</div>
  </div>
  <div class="input-row">
    <input type="text" id="user-input" placeholder="Type your question…" onkeydown="if(event.key==='Enter')send()">
    <button id="send-btn" onclick="send()">Send</button>
  </div>
</div>

<script>
const history = [];

async function send() {
  const input = document.getElementById('user-input');
  const btn = document.getElementById('send-btn');
  const msgs = document.getElementById('messages');
  const text = input.value.trim();
  if (!text) return;

  input.value = '';
  msgs.innerHTML += `<div class="msg user">${text}</div>`;
  msgs.innerHTML += `<div class="msg assistant" id="pending" style="color:var(--text-muted)">Thinking…</div>`;
  msgs.scrollTop = msgs.scrollHeight;

  btn.disabled = true;
  history.push({ role: 'user', content: text });

  try {
    const res = await fetch('https://api.anthropic.com/v1/messages', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model: 'claude-sonnet-4-6',
        max_tokens: 1000,
        system: 'You are a concise assistant for the Pauli ecosystem.',
        messages: history,
      }),
    });
    const data = await res.json();
    const reply = data.content.find(b => b.type === 'text')?.text || 'No response.';
    history.push({ role: 'assistant', content: reply });
    document.getElementById('pending').outerHTML = `<div class="msg assistant">${reply}</div>`;
  } catch {
    document.getElementById('pending').outerHTML = `<div class="msg assistant" style="color:var(--text-danger)">Request failed.</div>`;
  } finally {
    btn.disabled = false;
    msgs.scrollTop = msgs.scrollHeight;
  }
}
</script>
```

### Template C: Persistent data tracker
```html
<style>
  .tracker { padding: 1.5rem 0; }
  .entry { display: flex; align-items: center; gap: 10px; padding: 8px 12px; background: var(--surface-1); border-radius: var(--radius); margin-bottom: 6px; border: 0.5px solid var(--border); }
  .entry-label { flex: 1; font-size: 14px; }
  .entry-val { font-size: 14px; font-weight: 500; color: var(--text-accent); }
  .add-row { display: flex; gap: 8px; margin-bottom: 1rem; }
  .add-row input { flex: 1; }
</style>

<h2 style="position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0,0,0,0);">
  Persistent data tracker — entries saved across sessions
</h2>

<div class="tracker">
  <div class="add-row">
    <input type="text" id="new-label" placeholder="Entry name">
    <input type="number" id="new-val" placeholder="Value" style="width:100px">
    <button onclick="addEntry()">Add</button>
  </div>
  <div id="entries"><p style="color:var(--text-muted);font-size:14px">Loading…</p></div>
  <button onclick="sendPrompt('Analyze my tracked data and suggest improvements')" style="width:100%;margin-top:1rem;">
    Analyze with Claude ↗
  </button>
</div>

<script>
const STORAGE_KEY = 'tracker-entries';
let entries = [];

async function load() {
  try {
    const r = await window.storage.get(STORAGE_KEY);
    entries = r ? JSON.parse(r.value) : [];
  } catch { entries = []; }
  render();
}

async function addEntry() {
  const label = document.getElementById('new-label').value.trim();
  const val = parseFloat(document.getElementById('new-val').value);
  if (!label || isNaN(val)) return;

  entries.push({ id: Date.now(), label, val });
  document.getElementById('new-label').value = '';
  document.getElementById('new-val').value = '';

  try { await window.storage.set(STORAGE_KEY, JSON.stringify(entries)); } catch {}
  render();
}

async function removeEntry(id) {
  entries = entries.filter(e => e.id !== id);
  try { await window.storage.set(STORAGE_KEY, JSON.stringify(entries)); } catch {}
  render();
}

function render() {
  const el = document.getElementById('entries');
  if (!entries.length) {
    el.innerHTML = '<p style="color:var(--text-muted);font-size:14px">No entries yet.</p>';
    return;
  }
  el.innerHTML = entries.map(e => `
    <div class="entry">
      <span class="entry-label">${e.label}</span>
      <span class="entry-val">${Number.isInteger(e.val) ? e.val : e.val.toFixed(2)}</span>
      <button onclick="removeEntry(${e.id})" aria-label="Remove ${e.label}">
        <i class="ti ti-trash"></i>
      </button>
    </div>
  `).join('');
}

load();
</script>
```

---

## SECTION 14 — BANNED PATTERNS (ANTI-CHECKLIST)

These patterns appear frequently in AI-generated code and silently break artifacts.
Check each one before shipping.

```javascript
// ❌ localStorage — banned in sandbox
localStorage.setItem('k', v);

// ❌ CSS vars in canvas — ignored
ctx.fillStyle = 'var(--text-primary)';

// ❌ position: fixed — collapses iframe
el.style.position = 'fixed';

// ❌ External CDN not on allowlist
<script src="https://raw.githubusercontent.com/...">

// ❌ HTML/CSS comments — waste tokens, can break streaming
<!-- comment -->
/* comment */

// ❌ Hardcoded hex in CSS — invisible in dark mode
color: #333333;

// ❌ Font weight 600 or 700 — too heavy
font-weight: 700;

// ❌ display: none during initial render — streams invisibly
<div style="display:none" id="panel-2">

// ❌ Unrounded floats displayed to user
result.innerHTML = (a / b); // shows 0.30000000000000004

// ❌ Icon -filled suffix — not loaded, renders blank
<i class="ti ti-heart-filled">

// ❌ Dual y-axis chart — never use
scales: { y2: { position: 'right' } }

// ❌ Chart.js default legend — always disable
plugins: { legend: { display: true } }

// ❌ Vague language in UI text (Tablet I violation)
"Seamlessly integrates with your workflow"
"Robust performance"
"Cutting-edge AI"
```

---

## SECTION 15 — RALPHY LOOP OUTPUT

When this skill is executed by an agent in the Pauli ecosystem, the Ralphy Loop
requires a completion report in `ops/reports/`. Format:

```json
{
  "bead_id": "ARTIFACT-[ID]",
  "artifact_name": "descriptive name",
  "timestamp": "2026-06-29T00:00:00Z",
  "udec_score": 9.1,
  "axis_scores": {
    "sandbox_compliance": 10,
    "dark_mode_safety": 9,
    "accessibility": 8,
    "typography": 9,
    "number_formatting": 10,
    "layout_integrity": 9,
    "streaming_structure": 10,
    "interactivity_quality": 9,
    "data_visualization": 8,
    "emerald_tablets_compliance": 10
  },
  "libraries_used": ["Chart.js 4.4.1"],
  "storage_used": false,
  "api_used": false,
  "send_prompt_count": 3,
  "known_constraints_hit": [],
  "status": "shipped",
  "zero_context_handoff": true
}
```

---

## SUMMARY

An interactive artifact that ships correctly from this skill:

1. Reads this file before writing a single line
2. Respects all sandbox constraints (no localStorage, no fixed, CDN allowlist only)
3. Uses CSS variable tokens for all colors — dark mode works automatically
4. Reads CSS vars into JS for canvas and Chart.js
5. Puts style first, scripts last
6. Never uses display:none during initial render
7. Rounds all numbers before display
8. Has a screen reader landmark
9. Custom Chart.js legend, Tidepool palette colors in order
10. sendPrompt() on meaningful user actions with ↗ suffix
11. Error states on all async operations
12. Passes UDEC 8.5 self-audit before shipping
13. Produces Ralphy Loop JSON to ops/reports/

The artifact does one job. It does it correctly. It does not use vague language.
It works in dark mode. It scores ≥ 8.5. It does not ship until it does.

