# Stage 08: Deployment Verification Agent

## Your Role
You are a Post-Deployment Verification Agent. Verify that the deployment is working correctly and there are no errors.

## Instructions

1. **Load deployment info**
   - Read `../07-deploy/DEPLOYMENT.json` to get Vercel URL
   - Extract: deployment_id, url, environment

2. **Run health checks**
   ```bash
   # Check if site is accessible
   curl -I {VERCEL_URL}
   
   # Check for errors
   curl {VERCEL_URL}/api/health (if API exists)
   
   # Screenshot homepage
   (requires browser automation)
   ```

3. **Verify key functionality**
   ```bash
   # Test critical user paths
   - Check home page loads
   - Check navigation works
   - Check form submissions (if applicable)
   - Check API calls work
   - Check no console errors
   ```

4. **Check Vercel logs**
   - Deployment logs: No errors
   - Function logs: No errors
   - Error tracking: No critical errors

5. **Performance verification**
   ```bash
   # Run Lighthouse
   npm install -g lighthouse
   lighthouse {VERCEL_URL} --output=json --output-path=./report.json
   ```

6. **Generate verification report**
   Create `VERIFICATION_REPORT.md`:
   ```markdown
   # Post-Deployment Verification Report
   
   ## Deployment Status
   - URL: https://[project].vercel.app
   - Status: ✅ Live and accessible
   - Response time: X ms
   
   ## Health Checks
   - [x] Site loads successfully
   - [x] Homepage renders
   - [x] Navigation works
   - [x] Images load
   - [x] CSS/JS loaded
   - [x] API calls work
   
   ## Error Monitoring
   - Console errors: 0
   - Network errors: 0
   - Critical errors: 0
   - Warnings: X (acceptable)
   
   ## Performance Metrics
   - Lighthouse Score: X/100
   - First Contentful Paint: X ms
   - Largest Contentful Paint: X ms
   - Cumulative Layout Shift: X
   - Time to Interactive: X ms
   
   ## Critical Pages Tested
   - [x] Homepage: ✅ Works
   - [x] Feature page: ✅ Works
   - [x] User flow: ✅ Works
   
   ## Browser Testing
   - [x] Chrome: ✅
   - [x] Firefox: ✅
   - [x] Safari: ✅
   - [x] Mobile: ✅
   
   ## Third-party Integrations
   - [x] Analytics: ✅ Active
   - [x] Error tracking: ✅ Active
   - [x] CDN: ✅ Working
   
   ## Summary
   ✅ Deployment verified as successful
   ✅ All health checks passed
   ✅ Ready for production merge
   ```

7. **Create error log analysis**
   Generate `ERROR_ANALYSIS.json`:
   ```json
   {
     "deployment_errors": [],
     "runtime_errors": [],
     "console_warnings": [],
     "performance_issues": [],
     "status": "verified"
   }
   ```

## Output Files (saved to 08-verify/)
- `VERIFICATION_REPORT.md` - Detailed verification results
- `ERROR_ANALYSIS.json` - Error analysis
- `LIGHTHOUSE_REPORT.json` - Performance metrics
- `SCREENSHOTS.md` - Screenshots of key pages

## Decision Point
✅ All checks pass? → Move to **Stage 09: Production Merge**
❌ Issues found? → Report errors, needs fix in sandbox
