# ⛔ ACCESS CONTROL — BAMBU ONLY
# This file contains Bambu's personal second brain data.
# DO NOT load during Ivette/Kupuri sessions.
# DO NOT cross-reference with workspace/ivette/ content.
# If the current session context is Kupuri/Ivette, STOP and load
# workspace/ivette/brain/CONTEXT.md instead.

---

# Second Brain / Mind Mappy Context
# Load this when: accessing knowledge, mental models, second brain queries
# Owner: Bambu (Archon X ecosystem)

## Tools
- **Mind Mappy**: Primary second brain interface
- **Supabase**: `31.220.58.212:8001` — persistent storage, second brain DB
  - Database: `second_brain` (PostgreSQL)
  - Connection: `postgresql://postgres:***@31.220.58.212:5434/second_brain`
- **InfraNodus MCP**: `executiveusa/pauli-infranodus` — graph-based knowledge mapping
- **pauli-my-Brain-Is-Full-Crew**: AI memory augmentation system

## Mental Models Library
Source: `C:\Users\execu\Downloads\MIND MAPPY 2ND BRAIN\MENTAL MODELS.txt`
Key frameworks:
1. Dhandho — asymmetric bets, clone what works
2. Circle of competence — stay in known domains
3. Compounding — time × consistent improvement
4. Signal vs. noise — cut ruthlessly
5. Skin in the game — execute, don't just advise
6. Heads I win, tails I don't lose much
7. Low-hanging fruit — highest ROI first
8. Hire slow, fire fast (applies to sub-agents too)
9. Givers vs. takers
10. Rule of 72 — compounding timelines

## Graph Knowledge Access
- InfraNodus visualizes idea graphs from notes/text
- MCP commands: `query_graph`, `add_node`, `get_central_concepts`
- Use to find connections between projects, ideas, strategies

## Second Brain Query Protocol
1. Load context-mode MCP first (no raw output dumps)
2. Use `supabase-mcp` to query knowledge tables
3. Use InfraNodus to find graph connections
4. Summarize findings in ≤5 bullets before acting
