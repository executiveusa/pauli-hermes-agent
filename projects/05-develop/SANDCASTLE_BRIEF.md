# Stage 05: Development in Sandcastle

## Execution Method
This stage runs inside **Sandcastle** - an isolated sandbox with full git, npm, and execution capabilities.

## Your Mission

You are a full-stack developer. Your job is to implement the work plan from Stage 04 **exactly**.

### Phase 1: Environment Setup
```bash
# These run automatically in sandbox
git clone {REPO_URL} .
git checkout -b "feature/{slug}"
npm install
```

### Phase 2: Implement Changes

**Follow the implementation plan step by step:**

1. Read `../04-plan/IMPLEMENTATION_PLAN.md` completely
2. Read `../04-plan/TASK_CHECKLIST.json` 
3. Read `../03-design/IMPLEMENTATION_GUIDE.md`
4. Create/modify files exactly as specified
5. Check your work against the plan

**Code Rules:**
- Follow design system exactly (colors, spacing, components)
- Match the codebase's style (tabs/spaces, naming, structure)
- Write clean, readable code
- Add comments where logic is complex
- No console.log or debug code left in
- Follow TypeScript strictly if used

### Phase 3: Testing

After implementing each phase:
```bash
npm test  # Run unit tests
npm run lint  # Check code style
npm run build  # Test production build
```

**Your testing checklist (from plan):**
- [ ] All unit tests pass
- [ ] No console warnings/errors
- [ ] Build succeeds
- [ ] No unused imports

### Phase 4: Commit and Verify

After implementation:
```bash
git add .
git commit -m "Implement: {feature description}"

# Verify changes
git status
git log -1
```

## Expected Output

**Commit Messages:**
```
Implement: Add dark mode toggle component

- Create DarkModeToggle component
- Update Header to use toggle
- Add Tailwind dark mode classes
- Add unit tests for toggle logic
- All tests passing (42 passed)
```

**Files Created/Modified:**
- List all changed files with line counts
- All files follow design system

**Test Results:**
```
PASS  src/components/__tests__/DarkModeToggle.test.tsx
  DarkModeToggle
    ✓ renders toggle button (5ms)
    ✓ toggles dark mode on click (3ms)
    ✓ respects system preference (2ms)

Test Suites: 1 passed, 1 total
Tests: 3 passed, 3 total
```

## Sandcastle Safety

This runs in a **sandbox**:
- ✅ All changes are isolated
- ✅ Nothing touches production
- ✅ You can make mistakes safely
- ✅ Tests must pass before merge

## Success Criteria

Before marking complete:
```
☑️ All code changes implemented
☑️ All tests passing
☑️ npm run build succeeds
☑️ No console warnings/errors  
☑️ Commits are clean and descriptive
☑️ Design system compliance verified
☑️ Branch is ready to merge
```

## If Issues Arise

Problems during implementation?
- **Test failure**: Fix the code, re-run tests
- **Build error**: Check for TypeScript/syntax issues
- **Design mismatch**: Review design spec, fix styles
- **Dependency issue**: Check package versions, update if needed

## Next Step (Automatic)

After sandbox completes:
1. Tests are verified
2. Build is confirmed
3. Move to **Stage 06: Automated Testing**
4. Then **Stage 07: Vercel Deployment**

---

## Running Sandcastle

This will be triggered by:
```bash
hermes "sandbox the development work for project XYZ"
```

Or manually from Stage 04:
```bash
# Create a Sandcastle run using the implementation plan
```

The sandbox creates a feature branch, executes the plan, runs tests, and awaits your approval to merge.
