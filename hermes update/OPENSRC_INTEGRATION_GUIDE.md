# OPENSRC INTEGRATION FOR ARCHON X BACKEND
## Deep Dependency Context for Codex Agent

**Purpose**: Use OpenSrc to give Codex access to source code of all dependencies  
**Benefit**: Codex can understand implementation details, not just types/docs  
**Use Case**: When building Hermes integration, Codex can read actual agent code

---

## 🎯 WHY OPENSRC FOR THIS PROJECT

When Codex implements the Archon X backend, it needs to:
1. Understand FastAPI internals (streaming, async handling)
2. Understand SQLAlchemy ORM patterns (session management, RLS)
3. Understand Hermes agent structure (how it works internally)
4. Reference actual implementations of dependencies

**Without OpenSrc**: Codex has only type hints + documentation  
**With OpenSrc**: Codex has actual source code to reference

Example:
```
Codex needs to implement streaming responses
- Without OpenSrc: reads FastAPI docs
- With OpenSrc: can read how FastAPI does it, apply same patterns
```

---

## 📦 SETUP OPENSRC FOR ARCHON X

### Installation

```bash
# Global install
npm install -g opensrc

# Or use with npx (no install needed)
npx opensrc <package>
```

### Fetch Critical Dependencies

```bash
# Backend dependencies
opensrc fastapi
opensrc sqlalchemy
opensrc pydantic
opensrc python-jose

# Frontend dependencies  
opensrc react
opensrc next
opensrc @ai-sdk/react

# Testing
opensrc pytest
opensrc pytest-asyncio

# Fetch multiple at once
opensrc fastapi sqlalchemy pydantic python-jose pytest
```

### GitHub Repositories (Optional)

```bash
# Hermes agent (your fork)
opensrc executiveusa/pauli-hermes-agent

# Hermes upstream
opensrc NousResearch/hermes-agent

# Reference implementations
opensrc vercel/next.js
opensrc pallets/flask  # For streaming patterns
opensrc sqlalchemy/sqlalchemy
```

### After Setup

```bash
# List fetched sources
opensrc list

# This creates:
# opensrc/
# ├── settings.json
# ├── sources.json
# ├── fastapi/
# ├── sqlalchemy/
# ├── pydantic/
# └── ... (other packages)
```

---

## 🤖 USING OPENSRC WITH CODEX

### Tell Codex Where to Find Source

Add to SPEC.md or create `AGENTS.md`:

```markdown
# For Codex Agent

When implementing this backend, you have access to source code for all dependencies.

## Available Source Code

Use `opensrc list` to see all available sources.

Key files to reference:
- `opensrc/fastapi/` — understand streaming responses
- `opensrc/sqlalchemy/` — ORM patterns, session management
- `opensrc/pydantic/` — validation patterns
- `opensrc/python-jose/` — JWT implementation

## How to Use

When implementing a feature:
1. Check if there's source code: `opensrc list | grep <dependency>`
2. Reference the actual implementation in `opensrc/<package>/`
3. Apply similar patterns to your code
4. This ensures compatibility and best practices

## Example: Streaming Responses

Task: Implement streaming message endpoint
- Read: `opensrc/fastapi/fastapi/responses.py`
- Reference: How FastAPI implements StreamingResponse
- Apply: Same pattern for your /conversation/send endpoint
```

### Codex Commands

```bash
# Before implementing a feature
opensrc list

# If Codex needs to understand something
# Example: "I need to understand how FastAPI handles streaming"
opensrc fastapi

# Then reference in implementation
# Look at: opensrc/fastapi/fastapi/responses.py
```

---

## 🔍 KEY SOURCE FILES FOR ARCHON X

### FastAPI Streaming Implementation
```bash
opensrc/fastapi/fastapi/responses.py
opensrc/fastapi/fastapi/staticfiles.py
```

Reference for: `GET /conversation/send` endpoint streaming

### SQLAlchemy ORM Patterns
```bash
opensrc/sqlalchemy/sqlalchemy/orm/session.py
opensrc/sqlalchemy/sqlalchemy/orm/query.py
```

Reference for: Query builders, relationship management, RLS integration

### Pydantic Validation
```bash
opensrc/pydantic/pydantic/main.py
opensrc/pydantic/pydantic/validators.py
```

Reference for: API request/response schemas, input validation

### Python-Jose JWT
```bash
opensrc/python-jose/jose/__init__.py
opensrc/python-jose/jose/jwk.py
```

Reference for: JWT token generation/verification flow

### Hermes Agent Structure
```bash
opensrc/pauli-hermes-agent/agent/auxiliary_client.py
opensrc/pauli-hermes-agent/hermes_cli/claw.py
```

Reference for: How Hermes agent works, tool integration

---

## 📋 OPENSRC WORKFLOW FOR CODEX

### Week 1 Architecture Phase

```bash
# Codex needs to understand the dependencies
opensrc fastapi sqlalchemy pydantic

# Creates opensrc/ directory with source code
# Codex can now reference actual implementations
```

### Week 2-3 Backend Implementation

```bash
# When implementing auth endpoint
opensrc python-jose
# Reference: opensrc/python-jose/jose/ for JWT patterns

# When implementing database layer
opensrc sqlalchemy
# Reference: opensrc/sqlalchemy/sqlalchemy/orm/ for ORM patterns

# When implementing streaming
opensrc fastapi
# Reference: opensrc/fastapi/fastapi/responses.py for StreamingResponse
```

### Week 5-6 Frontend Implementation

```bash
# Frontend dependencies
opensrc react next @ai-sdk/react

# When implementing chat component
opensrc @ai-sdk/react
# Reference: opensrc/@ai-sdk/react/ for streaming patterns

# When implementing auth flow
opensrc next
# Reference: opensrc/next/packages/next-auth/ for patterns
```

---

## 🛠️ CODEX INTEGRATION INSTRUCTIONS

Add this to the build specification:

```markdown
## Using OpenSrc for Source Code Reference

This project uses OpenSrc to provide AI agents with access to dependency source code.

### Setup

```bash
# Install opensrc
npm install -g opensrc

# Fetch key dependencies
opensrc fastapi sqlalchemy pydantic python-jose pytest

# Check what's available
opensrc list
```

### When Implementing

1. **Before writing code**: Check if there's source available
2. **If implementing a pattern**: Reference the actual implementation
3. **Apply same patterns**: Use proven approaches from dependencies

### Examples

**Task: Implement streaming endpoint**
```bash
# Check available source
opensrc list | grep fastapi

# Read implementation
cat opensrc/fastapi/fastapi/responses.py

# Apply pattern to your code
```

**Task: Implement ORM queries**
```bash
# Check available source
opensrc list | grep sqlalchemy

# Read ORM patterns
cat opensrc/sqlalchemy/sqlalchemy/orm/session.py

# Apply to your models
```

**Task: Implement JWT auth**
```bash
# Check available source
opensrc list | grep python-jose

# Read JWT implementation
cat opensrc/python-jose/jose/__init__.py

# Apply to your auth module
```

### Benefits

- **Proven patterns**: Reference how professionals implement features
- **Compatibility**: Ensure your code integrates well with dependencies
- **Learning**: Understand how complex libraries work internally
- **Problem solving**: When stuck, check how the dependency handles similar cases
```

---

## 📊 PROJECT OPENSRC STRUCTURE

After setup, project structure:

```
archon-x-backend/
├── src/
│   ├── main.py
│   ├── agents.py
│   ├── auth.py
│   └── ...
├── frontend/
│   ├── app/
│   └── ...
├── opensrc/                    ← NEW (added by opensrc)
│   ├── settings.json
│   ├── sources.json
│   ├── fastapi/                ← FastAPI source code
│   ├── sqlalchemy/             ← SQLAlchemy source code
│   ├── pydantic/               ← Pydantic source code
│   ├── python-jose/            ← JWT library source
│   ├── pytest/                 ← Testing framework
│   ├── pauli-hermes-agent/     ← Hermes agent (if fetched)
│   └── ...
├── .gitignore                  ← Updated to ignore opensrc/
├── tsconfig.json               ← Updated to exclude opensrc/
├── AGENTS.md                   ← Updated with opensrc section
└── ...
```

### .gitignore Addition

```bash
# OpenSrc - dependency source code for agent reference
opensrc/
```

### tsconfig.json Addition

```json
{
  "exclude": ["opensrc"]
}
```

### AGENTS.md Creation

Created automatically with:
```markdown
# Agent Context

Source code for key dependencies is available in `opensrc/` directory.

See opensrc/sources.json for complete list.

Agents should reference source code when implementing features.
```

---

## 🎯 CODEX USAGE PATTERNS

### Pattern 1: Reference Implementation

```
Codex: "I need to implement streaming responses"
↓
Check: opensrc list | grep fastapi
↓
Reference: opensrc/fastapi/fastapi/responses.py
↓
Apply: Same pattern to /conversation/send endpoint
```

### Pattern 2: Understand Integration

```
Codex: "How do I integrate SQLAlchemy ORM with async FastAPI?"
↓
Check: opensrc/sqlalchemy/ and opensrc/fastapi/
↓
Read: session management patterns in both
↓
Apply: Best practices from both libraries
```

### Pattern 3: Debug Issues

```
Codex: "Why isn't my streaming working?"
↓
Check: opensrc/fastapi/fastapi/responses.py
↓
Compare: Your implementation vs. reference
↓
Fix: Align with proven patterns
```

---

## ⚙️ OPENSRC MANAGEMENT

### Keep Updated

```bash
# Refresh to latest versions
opensrc fastapi sqlalchemy pydantic

# Automatically updates to match your package.json versions
```

### List and Clean

```bash
# See what you have
opensrc list

# Remove a package
opensrc remove fastapi

# Remove all
opensrc remove --all
```

### Settings

```bash
# View settings
cat opensrc/settings.json

# Modify settings (prevents re-prompts)
{
  "allowFileModifications": true
}
```

---

## 📚 SOURCE FILES TO REVIEW

### Critical for This Project

1. **FastAPI Streaming**
   ```
   opensrc/fastapi/fastapi/responses.py
   opensrc/fastapi/fastapi/staticfiles.py
   ```
   → For Server-Sent Events streaming

2. **SQLAlchemy ORM**
   ```
   opensrc/sqlalchemy/sqlalchemy/orm/session.py
   opensrc/sqlalchemy/sqlalchemy/orm/query.py
   opensrc/sqlalchemy/sqlalchemy/pool/
   ```
   → For connection pooling, RLS integration

3. **Pydantic Validation**
   ```
   opensrc/pydantic/pydantic/main.py
   opensrc/pydantic/pydantic/validators.py
   ```
   → For input/output validation

4. **JWT/Security**
   ```
   opensrc/python-jose/jose/__init__.py
   opensrc/python-jose/jose/jwk.py
   opensrc/python-jose/jose/backends/
   ```
   → For token generation/verification

5. **Hermes Agent** (if fetched)
   ```
   opensrc/pauli-hermes-agent/agent/
   opensrc/pauli-hermes-agent/hermes_cli/
   ```
   → For agent integration patterns

---

## ✅ OPENSRC CHECKLIST FOR CODEX

- [ ] Install opensrc: `npm install -g opensrc`
- [ ] Fetch dependencies: `opensrc fastapi sqlalchemy pydantic python-jose pytest`
- [ ] Verify setup: `opensrc list` shows all packages
- [ ] Review AGENTS.md (auto-generated)
- [ ] Check .gitignore (auto-updated)
- [ ] Reference source code when implementing features
- [ ] Compare your implementations with reference
- [ ] Keep updated: re-run opensrc periodically

---

## 🚀 INTEGRATION WITH BUILD SPEC

When giving the CODEX_BUILD_SPECIFICATION_COMPLETE.md to Codex, include:

1. The main spec document
2. This OpenSrc integration guide
3. Quick start command:

```bash
# Get everything set up
npm install -g opensrc
opensrc fastapi sqlalchemy pydantic python-jose pytest
```

4. Reference in AGENTS.md section of build spec:

```markdown
## Using OpenSrc for Reference Implementations

All key dependencies have their source code available in the `opensrc/` directory.

When implementing a feature, check the reference implementation in opensrc:
- Streaming: See opensrc/fastapi/fastapi/responses.py
- ORM patterns: See opensrc/sqlalchemy/sqlalchemy/orm/
- Validation: See opensrc/pydantic/pydantic/
- Auth: See opensrc/python-jose/jose/

This ensures your implementation aligns with proven patterns.
```

---

## 💡 WHY THIS MATTERS FOR CODEX

**Without OpenSrc**:
- Codex reads FastAPI docs
- Codex reads SQLAlchemy docs
- Codex guesses best patterns
- Result: Decent code, but not optimal

**With OpenSrc**:
- Codex reads actual source code
- Codex sees proven implementations
- Codex applies same patterns
- Result: Production-quality code, aligned with dependencies

This is especially valuable for complex features like:
- Streaming responses (Server-Sent Events)
- Database connection pooling
- Async/await patterns
- Error handling

---

**OpenSrc bridges the gap between "I know the API" and "I understand how it works."**

For Codex implementing the Archon X backend, this is critical for quality.

Setup takes 5 minutes. Value adds 10x to implementation quality.

Do it first. Reference often.
