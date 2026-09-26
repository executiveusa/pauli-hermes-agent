# ICM Form Selector

Use this to choose the right ICM form for your workspace. **Ask:** What is the repeating unit of work?

## The Five Forms

### 1. Pipeline — The Production Line

**Use when:** The same sequence runs repeatedly (daily, weekly, per-request), producing a deliverable each time.

**The repeating unit:** A run (article, video, analysis, report, customer request).

**Shape:**
```
workspace/
├─ CLAUDE.md
├─ CONTEXT.md
├─ stages/
│  ├─ 01_[stage]/
│  ├─ 02_[stage]/
│  └─ 03_[stage]/
├─ _shared/
└─ _templates/
```

**Example:** Content studio (research → script → production → delivery)

**Defining move:** One stage's `output/` is the next stage's input. Humans edit at each boundary.

---

### 2. Umbrella — Portfolio of Pipelines

**Use when:** You have several distinct production lines that share brand, voice, and reference material.

**The repeating unit:** Different kinds of runs, each with its own pipeline.

**Shape:**
```
workspace/
├─ CLAUDE.md (the map: which pipeline for which task)
├─ 01_reference/
├─ 02_brand-voice/
├─ 03_pipeline-A/ (full Pipeline workspace)
├─ 04_pipeline-B/ (full Pipeline workspace)
└─ 05_pipeline-C/ (full Pipeline workspace)
```

**Example:** Production company doing videos, graphics, and animations (separate pipelines, shared brand).

**Defining move:** Root maps tasks to pipelines. Sub-pipelines are self-contained. Reference is shared.

---

### 3. Record Library — The Unit Is a Record

**Use when:** Nothing "runs" to completion; records (people, clients, sessions, projects) get created, accumulate, and are looked up.

**The repeating unit:** A record (not a run).

**Shape:**
```
workspace/
├─ 00_START-HERE.md
├─ _index/log.md (one line per record)
├─ _templates/
│  └─ record-template/
├─ 01_reference/
└─ records/
   ├─ [record-1]/
   └─ [record-2]/
```

**Example:** CRM (customers), contact manager (people), session tracker (coaching clients).

**Defining move:** New record = copy template, not blank page. Template is the schema. Index log is the source of truth.

---

### 4. Knowledge Bundle — The Product Is Knowledge

**Use when:** The deliverable is navigable knowledge itself: a brain, domain wiki, or model of a subject.

**The repeating unit:** A piece of knowledge (note, concept, connection).

**Shape:**
```
workspace/
├─ CLAUDE.md
├─ corpus/ (raw sources + manifest)
├─ extraction/ (factory: pipeline that builds the bundle)
└─ bundle/ (product: the navigable brain)
   ├─ index.md
   ├─ layers/
   └─ graph/ (connections)
```

**Example:** Second brain (Obsidian-style), domain expertise vault, product knowledge base.

**Defining move:** Factory (pipeline that extracts/refines) is separate from product (the brain itself). Knowledge is layered and interconnected.

---

### 5. Context Map — Organization as a Graph

**Use when:** The subject is an organization: teams, processes, data, and the relationships between them.

**The repeating unit:** An entity (team, process, person, data source) and its relationships.

**Shape:**
```
workspace/
├─ CLAUDE.md (the org map)
├─ org-map.md (graph definition)
├─ entities/ (teams, processes, people, services)
│  ├─ [entity-1].md
│  └─ [entity-2].md
├─ relationships/ (who touches whom, data flows, dependencies)
└─ _reference/ (org-wide standards)
```

**Example:** Company structure, team dependencies, service mesh, integration map.

**Defining move:** Identity is relationships. Entities are points; edges are the work. Querying the map answers "who talks to whom" and "how does X flow through the org."

---

## Mixed Forms

Real workspaces often compose forms:

- **Record library + Knowledge bundle inside each record:** Each customer-record is its own mini brain.
- **Umbrella + Context map:** Portfolio of pipelines organized by the org structure.
- **Pipeline → Record library:** A production line that emits into an accumulating library.

**The invariants hold at every level. Apply them recursively.**

---

## Decision Tree

**Q: Does the same sequence run repeatedly with new deliverables?**
- Yes → **Pipeline**
- No, go to Q2

**Q: Do you have several distinct sequences sharing one brand/identity?**
- Yes → **Umbrella**
- No, go to Q3

**Q: Is the unit a record that accumulates (person, client, session)?**
- Yes → **Record library**
- No, go to Q4

**Q: Is the product knowledge itself — a brain, wiki, or model?**
- Yes → **Knowledge bundle**
- No, go to Q5

**Q: Are you mapping an organization or system (teams, processes, data)?**
- Yes → **Context map**
- No → Rethink the repeating unit. You might not need ICM (just a saved prompt), or the unit isn't clear yet.
