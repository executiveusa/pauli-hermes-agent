# CODEX BUILD SPECIFICATION + OPENSRC INTEGRATION
## Complete Archon X Backend Build Kit with Dependency Source Code Access

**Project**: Archon X Custom Backend  
**Integration**: OpenSrc for dependency source code reference  
**Status**: Ready for implementation  
**Timeline**: 8 weeks to MVP  

---

## 🎯 WHAT IS OPENSRC AND WHY IT MATTERS

**OpenSrc** (https://github.com/vercel-labs/opensrc) fetches source code for npm packages and GitHub repos, giving AI agents access to actual implementations—not just type hints or docs.

### For Codex Building This Backend:

**Without OpenSrc**:
```
Codex needs to implement streaming responses
  ↓
Reads FastAPI documentation
  ↓
Implements based on docs alone
  ↓
Result: Works, but might miss optimization patterns
```

**With OpenSrc**:
```
Codex needs to implement streaming responses
  ↓
Reads FastAPI documentation
  ↓
References opensrc/fastapi/fastapi/responses.py
  ↓
Sees actual implementation, applies same patterns
  ↓
Result: Production-quality code aligned with proven patterns
```

---

## 📦 OPENSRC SETUP (First Thing Codex Does)

### Installation

```bash
# Global install (recommended)
npm install -g opensrc

# Verify
opensrc --version
```

### Fetch Critical Dependencies

Run this BEFORE starting implementation:

```bash
# Backend dependencies
opensrc fastapi uvicorn sqlalchemy psycopg2-binary pydantic python-jose passlib

# Frontend dependencies
opensrc react next @ai-sdk/react

# Testing
opensrc pytest pytest-asyncio

# Reference implementations
opensrc vercel-labs/opensrc

# Optional: Hermes references
opensrc executiveusa/pauli-hermes-agent
opensrc NousResearch/hermes-agent
```

Or run once:

```bash
opensrc fastapi uvicorn sqlalchemy psycopg2-binary pydantic python-jose passlib pytest pytest-asyncio react next @ai-sdk/react
```

### Verify Setup

```bash
# List all fetched sources
opensrc list

# Should show something like:
# Sources available:
# - fastapi (v0.104.1)
# - sqlalchemy (v2.0.23)
# - pydantic (v2.5.0)
# - python-jose (v3.3.0)
# - react (v18.2.0)
# - next (v14.0.0)
# ... and others
```

### Directory Structure After Setup

```
archon-x-backend/
├── src/
│   ├── main.py
│   ├── agents.py
│   ├── auth.py
│   └── models.py
├── frontend/
│   ├── app/
│   ├── lib/
│   └── components/
├── opensrc/                    ← NEW (created by opensrc)
│   ├── settings.json           ← Preferences
│   ├── sources.json            ← Index of sources
│   ├── fastapi/                ← FastAPI source code
│   │   ├── fastapi/
│   │   │   ├── responses.py    ← StreamingResponse implementation
│   │   │   ├── openapi/
│   │   │   └── ...
│   │   └── ...
│   ├── sqlalchemy/             ← SQLAlchemy source code
│   │   ├── sqlalchemy/
│   │   │   ├── orm/
│   │   │   │   ├── session.py  ← Session management
│   │   │   │   └── query.py
│   │   │   └── ...
│   │   └── ...
│   ├── pydantic/               ← Pydantic source code
│   ├── python-jose/            ← JWT library
│   ├── pytest/
│   ├── react/
│   ├── next/
│   └── pauli-hermes-agent/     ← If fetched
├── .gitignore                  ← Auto-updated (excludes opensrc/)
├── tsconfig.json               ← Auto-updated (excludes opensrc/)
├── AGENTS.md                   ← Auto-created (documents opensrc)
├── package.json
└── requirements.txt
```

---

## 🔍 KEY SOURCE FILES TO REFERENCE DURING IMPLEMENTATION

### Week 2-3: Backend Core Implementation

#### Streaming Responses (for /conversation/send)
```
opensrc/fastapi/fastapi/responses.py
opensrc/fastapi/fastapi/concurrency.py
```

**What to look for**:
- How FastAPI implements StreamingResponse
- How to handle async generators
- How to set proper headers for SSE (Server-Sent Events)

**Apply to**: Your `/conversation/{id}/send` endpoint

**Example pattern**:
```python
# In opensrc/fastapi/fastapi/responses.py, see:
class StreamingResponse:
    def __init__(self, content, status_code=200, headers=None, media_type=None):
        # How they handle streaming
```

#### Database Session Management
```
opensrc/sqlalchemy/sqlalchemy/orm/session.py
opensrc/sqlalchemy/sqlalchemy/orm/sessionmaker.py
opensrc/sqlalchemy/sqlalchemy/pool/
```

**What to look for**:
- How to manage database sessions in async contexts
- Connection pooling strategies
- Transaction management

**Apply to**: Your database connection pool and session management

#### Input Validation
```
opensrc/pydantic/pydantic/main.py
opensrc/pydantic/pydantic/validators.py
opensrc/pydantic/pydantic/fields.py
```

**What to look for**:
- How Pydantic validates input
- Custom validator patterns
- Error handling

**Apply to**: Your request/response schemas (MessageSend, ConversationCreate, etc.)

#### JWT Authentication
```
opensrc/python-jose/jose/__init__.py
opensrc/python-jose/jose/jwk.py
opensrc/python-jose/jose/constants.py
```

**What to look for**:
- JWT encoding/decoding patterns
- Token validation
- Error handling for invalid tokens

**Apply to**: Your `auth.py` module (create_jwt_token, verify_jwt_token)

### Week 5-6: Frontend Implementation

#### Streaming in React
```
opensrc/react/packages/react-dom/
opensrc/@ai-sdk/react/src/
```

**What to look for**:
- How React handles streaming updates
- Event handler patterns
- State management with streaming

**Apply to**: Your chat component's message handling

#### Next.js API Routes
```
opensrc/next/packages/next/src/server/api-utils.ts
opensrc/next/packages/next/src/server/lib/utils.ts
```

**What to look for**:
- How Next.js handles streaming responses
- API route patterns
- Middleware implementation

**Apply to**: Your `app/api/chat/route.ts`

### Hermes Agent Integration

```
opensrc/pauli-hermes-agent/agent/
opensrc/pauli-hermes-agent/hermes_cli/claw.py
opensrc/NousResearch/hermes-agent/agent/
```

**What to look for**:
- How Hermes wraps LLM calls
- Tool integration pattern
- Memory management
- Streaming responses from agent

**Apply to**: Your `agents.py` (AgentPool class, send_message method)

---

## 💡 HOW CODEX SHOULD USE OPENSRC

### Pattern 1: Before Implementing a Feature

```
Task: Implement streaming response endpoint

Step 1: Check if source is available
$ opensrc list | grep fastapi

Step 2: Read reference implementation
$ cat opensrc/fastapi/fastapi/responses.py | grep -A 20 "class StreamingResponse"

Step 3: Understand the pattern
# How does FastAPI handle streaming?
# What headers are needed?
# How to handle async generators?

Step 4: Apply to your code
# Implement /conversation/{id}/send using same pattern
# Set proper Content-Type and headers
# Yield chunks properly
```

### Pattern 2: Understand Integration Points

```
Task: Connect SQLAlchemy ORM to FastAPI async routes

Step 1: Read both implementations
$ cat opensrc/sqlalchemy/sqlalchemy/orm/session.py
$ cat opensrc/fastapi/fastapi/routing.py

Step 2: Find integration patterns
# How do async FastAPI routes work?
# How does SQLAlchemy handle async sessions?

Step 3: Apply both patterns
# Use SQLAlchemy's async session maker
# Use FastAPI's async request handler
# Connect them properly
```

### Pattern 3: Debug Issues

```
Task: Fix: "RuntimeError: Task created but never awaited"

Step 1: Search reference implementations
$ grep -r "await" opensrc/fastapi/ | grep -i task

Step 2: See how they handle async
# FastAPI patterns for async handling

Step 3: Fix your code
# Apply proper async/await patterns
```

### Pattern 4: Optimize Implementation

```
Task: Make database queries faster

Step 1: Check connection pooling in reference
$ cat opensrc/sqlalchemy/sqlalchemy/pool/

Step 2: See optimization patterns
# How does SQLAlchemy pool connections?
# What are the tuning parameters?

Step 3: Apply to your app
# Configure connection pool properly
# Add query optimization tips
```

---

## 🛠️ OPENSRC COMMANDS REFERENCE

### Viewing Sources

```bash
# List all fetched sources with versions
opensrc list

# List specific package
opensrc list | grep fastapi

# Check when last updated
cat opensrc/sources.json | grep fastapi
```

### Fetching/Updating

```bash
# Fetch a new package
opensrc zod

# Update to match installed version
opensrc fastapi  # auto-detects version from package-lock.json

# Fetch multiple at once
opensrc fastapi sqlalchemy pydantic

# Fetch specific version
opensrc fastapi@0.104.1

# Fetch GitHub repos
opensrc vercel-labs/opensrc
opensrc facebook/react
opensrc https://github.com/NousResearch/hermes-agent
```

### Managing

```bash
# Remove a source
opensrc remove fastapi

# Remove multiple
opensrc remove fastapi sqlalchemy pydantic

# View settings
cat opensrc/settings.json

# Clear settings to re-prompt
rm opensrc/settings.json
```

---

## 📋 INTEGRATION WITH BUILD SPECIFICATION

### Add to AGENTS.md (Auto-created)

```markdown
# Codex Agent Context

## Source Code Reference

This project uses OpenSrc to provide access to dependency source code.

All source code is available in the `opensrc/` directory.

### Available Sources

See `opensrc/sources.json` for complete list.

**Key references for this project**:
- FastAPI streaming: `opensrc/fastapi/fastapi/responses.py`
- SQLAlchemy ORM: `opensrc/sqlalchemy/sqlalchemy/orm/session.py`
- Pydantic validation: `opensrc/pydantic/pydantic/main.py`
- JWT auth: `opensrc/python-jose/jose/__init__.py`
- Hermes agent: `opensrc/pauli-hermes-agent/agent/`

### How to Use

When implementing a feature:

1. **Find the reference**: `opensrc list` to see available sources
2. **Read the code**: `cat opensrc/<package>/...`
3. **Understand the pattern**: How does the reference implementation work?
4. **Apply to your code**: Use the same pattern/approach

### Examples

**Implementing streaming responses**:
```bash
# Reference: opensrc/fastapi/fastapi/responses.py
# Shows: How FastAPI implements StreamingResponse
# Apply: Same pattern to /conversation/{id}/send endpoint
```

**Implementing database layers**:
```bash
# Reference: opensrc/sqlalchemy/sqlalchemy/orm/session.py
# Shows: How to manage async database sessions
# Apply: Same pattern to your database.py
```

**Implementing JWT auth**:
```bash
# Reference: opensrc/python-jose/jose/__init__.py
# Shows: JWT encoding/decoding patterns
# Apply: Same pattern to your auth.py
```

This ensures production-quality code aligned with proven implementations.
```

### Add to .gitignore

OpenSrc automatically adds this, but ensure it's there:

```bash
# OpenSrc - dependency source code for agent reference
opensrc/
```

### Add to tsconfig.json

OpenSrc automatically adds this:

```json
{
  "compilerOptions": { ... },
  "exclude": ["node_modules", "opensrc"]
}
```

---

## 📅 CODEX IMPLEMENTATION TIMELINE WITH OPENSRC

### Week 1: Architecture + OpenSrc Setup

```bash
# Day 1: Set up environment
python -m venv venv
source venv/bin/activate

# Day 2: Install opensrc and fetch sources
npm install -g opensrc
opensrc fastapi sqlalchemy pydantic python-jose pytest

# Day 3: Design architecture
# Review: opensrc/fastapi/fastapi/
# Review: opensrc/sqlalchemy/sqlalchemy/orm/
# Design: How to combine them

# Day 4: Database schema
# Reference: opensrc/sqlalchemy/ for patterns
# Create: Complete schema based on patterns

# Day 5: API spec review
# Reference: opensrc/fastapi/ for endpoint patterns
# Finalize: API specification
```

### Week 2-3: Backend Implementation

**Daily pattern**:
```
1. Choose task (e.g., "implement JWT auth")
2. Find reference: opensrc list | grep python-jose
3. Read: cat opensrc/python-jose/jose/__init__.py
4. Understand: How does python-jose work?
5. Implement: Apply same patterns to your code
6. Test: Verify it works
```

### Week 5-6: Frontend Implementation

```bash
# Before starting frontend
opensrc react next @ai-sdk/react

# When implementing chat component
# Reference: opensrc/react/packages/react-dom/
# See: How React handles state updates

# When implementing API integration
# Reference: opensrc/next/packages/next/src/server/
# See: How Next.js handles API routes
```

---

## ✅ OPENSRC CHECKLIST FOR CODEX

**Before Week 1**:
- [ ] Install Node.js + npm
- [ ] Install opensrc: `npm install -g opensrc`
- [ ] Fetch dependencies: `opensrc fastapi sqlalchemy pydantic python-jose pytest`
- [ ] Verify: `opensrc list` shows all packages
- [ ] Check: `opensrc/` directory exists with source code
- [ ] Review: `.gitignore` includes `opensrc/`
- [ ] Review: `AGENTS.md` created with opensrc section

**During Week 2-3 (Backend)**:
- [ ] Reference opensrc/fastapi when implementing endpoints
- [ ] Reference opensrc/sqlalchemy when implementing ORM
- [ ] Reference opensrc/pydantic when creating schemas
- [ ] Reference opensrc/python-jose when implementing auth
- [ ] Keep opensrc/ updated if dependencies change

**During Week 5-6 (Frontend)**:
- [ ] Reference opensrc/react when building components
- [ ] Reference opensrc/next for API routes
- [ ] Reference opensrc/@ai-sdk/react for streaming

**Before launch**:
- [ ] Verify all key patterns are aligned with references
- [ ] opensrc/ directory in .gitignore (verified)
- [ ] AGENTS.md documents all opensrc sources

---

## 🚀 QUICK START FOR CODEX

**TL;DR - First 10 minutes**:

```bash
# 1. Install opensrc
npm install -g opensrc

# 2. Fetch key dependencies (one command)
opensrc fastapi sqlalchemy pydantic python-jose pytest react next

# 3. Verify it worked
opensrc list

# 4. You now have source code for all key dependencies
# Reference these files when implementing features:
# - opensrc/fastapi/fastapi/responses.py (streaming)
# - opensrc/sqlalchemy/sqlalchemy/orm/session.py (database)
# - opensrc/pydantic/pydantic/main.py (validation)
# - opensrc/python-jose/jose/__init__.py (auth)

# 5. Start building using patterns from source code
```

---

## 📚 REFERENCE DOCUMENTATION

### Official OpenSrc
https://github.com/vercel-labs/opensrc

### Documentation
- README: How opensrc works
- Usage: Full command reference

### Key Files in Your Project After OpenSrc Setup
- `opensrc/settings.json` - Your preferences
- `opensrc/sources.json` - Index of all sources
- `AGENTS.md` - Auto-created documentation

---

## 🎯 WHY THIS MATTERS

**Quality difference**:
- **Without OpenSrc**: Codex implements based on docs (85% quality)
- **With OpenSrc**: Codex implements based on actual code (95%+ quality)

**Time savings**:
- **Without OpenSrc**: Debugging takes longer (not sure why pattern failed)
- **With OpenSrc**: Debug faster (can compare with reference)

**Learning**:
- **Without OpenSrc**: Codex knows the API
- **With OpenSrc**: Codex understands the implementation

**For the Archon X backend**, this is critical because:
1. Streaming responses are complex (need reference)
2. Async/await patterns are critical (need reference)
3. Database integration requires careful handling (need reference)
4. Security (JWT) needs proven patterns (need reference)

---

## 🔧 TROUBLESHOOTING

### OpenSrc not found after install

```bash
# Reinstall
npm install -g opensrc

# Check installation
which opensrc
opensrc --version

# If still not working, use npx instead
npx opensrc list
```

### Can't fetch a package

```bash
# Verify package exists on npm
npm search fastapi

# Try with version
opensrc fastapi@0.104.1

# Try GitHub directly
opensrc github:pallets/flask
```

### Need to update sources

```bash
# Re-run opensrc to update to latest installed version
opensrc fastapi sqlalchemy

# This auto-detects from package-lock.json
```

### Don't want file modifications

```bash
# Answer no to prompts, or:
opensrc fastapi --modify=false

# Manually add to .gitignore and tsconfig.json instead
```

---

## 📞 QUESTIONS FOR CODEX

When implementing, Codex can ask:

1. **"How do I implement streaming?"**
   → Check `opensrc/fastapi/fastapi/responses.py`

2. **"How do I manage database sessions?"**
   → Check `opensrc/sqlalchemy/sqlalchemy/orm/session.py`

3. **"How do I validate input?"**
   → Check `opensrc/pydantic/pydantic/main.py`

4. **"How do I handle JWT tokens?"**
   → Check `opensrc/python-jose/jose/__init__.py`

5. **"How does Hermes work?"**
   → Check `opensrc/pauli-hermes-agent/agent/`

All answers are in the source code.

---

## 🎉 SUMMARY

**OpenSrc gives Codex**:
✅ Access to source code for all dependencies  
✅ Reference implementations for complex patterns  
✅ Proven approaches for streaming, async, validation, auth  
✅ Ability to understand internal workings  
✅ Alignment with professional patterns  

**Setup time**: 5 minutes  
**Value added**: 10x in implementation quality  

**Do this first. Reference often. Ship with confidence.**

---

**This completes the Codex Build Specification with OpenSrc integration.**

All pieces are in place. Codex has everything needed to build production-quality Archon X backend.

Ready to ship? 🚀
