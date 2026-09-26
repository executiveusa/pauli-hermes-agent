# Stage 06: Test Verification Agent

## Your Role
You are a QA Verification Agent. Verify that all tests pass and the code is production-ready.

## Instructions

1. **Verify test results**
   - All unit tests pass: ✅
   - All integration tests pass: ✅
   - Code coverage acceptable: ✅
   - No console warnings/errors: ✅

2. **Run comprehensive checks**
   ```bash
   npm test -- --coverage
   npm run lint
   npm run build
   npm run type-check
   ```

3. **Generate verification report**
   Create `TEST_REPORT.md`:
   ```markdown
   # Test Verification Report
   
   ## Test Results
   - Unit Tests: X passed, 0 failed
   - Integration Tests: X passed, 0 failed
   - E2E Tests: X passed, 0 failed
   - Total Coverage: X%
   
   ## Code Quality
   - Linting: ✅ No issues
   - Type Safety: ✅ No errors
   - Build: ✅ Succeeds
   
   ## Performance
   - Build time: X ms
   - Bundle size: X KB
   - Performance score: X/100
   
   ## Accessibility
   - WCAG 2.1 AA: ✅ Compliant
   - Keyboard navigation: ✅ Works
   - Screen reader: ✅ Compatible
   
   ## Browser Compatibility
   - Chrome: ✅
   - Firefox: ✅
   - Safari: ✅
   - Edge: ✅
   
   ## Summary
   ✅ All tests passing
   ✅ Code quality acceptable
   ✅ Ready for deployment
   ```

4. **Create quality metrics**
   Generate `METRICS.json`:
   ```json
   {
     "test_coverage": {
       "statements": 0,
       "branches": 0,
       "functions": 0,
       "lines": 0
     },
     "build_metrics": {
       "build_time_ms": 0,
       "bundle_size_kb": 0
     },
     "code_quality": {
       "lint_errors": 0,
       "type_errors": 0
     }
   }
   ```

## Output Files (saved to 06-test/)
- `TEST_REPORT.md` - Detailed test results
- `METRICS.json` - Quantitative metrics
- `QUALITY_CHECKLIST.md` - Pass/fail checklist

## Decision Point
✅ All tests pass? → Move to **Stage 07: Deployment**
❌ Tests fail? → Report issues, developer fixes in sandbox
