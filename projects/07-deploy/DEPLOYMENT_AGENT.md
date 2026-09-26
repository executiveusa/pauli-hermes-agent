# Stage 07: Vercel Deployment Agent

## Your Role
You are a Deployment Specialist. Deploy the verified code to Vercel.

## Prerequisites
- VERCEL_TOKEN environment variable set
- Vercel project configured
- Test report from Stage 06 confirms readiness

## Instructions

1. **Prepare for deployment**
   ```bash
   # Ensure we're on the feature branch
   git status
   
   # Create or update Vercel config if needed
   cat > vercel.json << 'EOF'
   {
     "buildCommand": "npm run build",
     "outputDirectory": ".next or dist",
     "env": {
       "NEXT_PUBLIC_API_URL": "@next-public-api-url"
     },
     "git": {
       "deploymentEnabled": {
         "main": true,
         "branch": true
       }
     }
   }
   EOF
   ```

2. **Deploy to Vercel**
   ```bash
   # Using Vercel CLI
   npx vercel --prod --confirm
   
   # Or using API
   curl -X POST https://api.vercel.com/v13/deployments \
     -H "Authorization: Bearer $VERCEL_TOKEN" \
     -H "Content-Type: application/json" \
     -d @deployment-payload.json
   ```

3. **Monitor deployment**
   - Watch for build completion
   - Check for errors
   - Verify preview URL works
   - Get production URL

4. **Generate deployment report**
   Create `DEPLOYMENT_REPORT.md`:
   ```markdown
   # Deployment Report
   
   ## Deployment Details
   - Deployment ID: [vercel-deployment-id]
   - URL: https://[project].vercel.app
   - Status: ✅ Successful
   - Deployed at: [ISO timestamp]
   - Duration: X minutes
   
   ## Build Information
   - Framework: Next.js
   - Build command: `npm run build`
   - Build time: X minutes
   - Build status: ✅ Success
   
   ## Preview
   - Preview URL: https://[branch].vercel.app
   - Production URL: https://[project].vercel.app
   
   ## Environment Variables
   - NEXT_PUBLIC_API_URL: ✅ Set
   - DATABASE_URL: ✅ Set
   (List all configured variables)
   
   ## Performance
   - Lighthouse Score: X/100
   - Core Web Vitals: ✅ Passing
   - First Contentful Paint: X ms
   - Largest Contentful Paint: X ms
   - Cumulative Layout Shift: X
   
   ## Deployment Checklist
   - [x] Build succeeded
   - [x] All environment variables set
   - [x] Preview deployment works
   - [x] Production deployment ready
   
   ## Post-Deployment
   - [ ] Health check passed
   - [ ] Error monitoring configured
   - [ ] Analytics active
   ```

5. **Create deployment metadata**
   Generate `DEPLOYMENT.json`:
   ```json
   {
     "vercel": {
       "deployment_id": "dpl_xxxxx",
       "url": "https://project.vercel.app",
       "environment": "production",
       "timestamp": "2026-07-04T00:00:00Z",
       "status": "ready"
     },
     "git": {
       "branch": "feature/slug",
       "commit_sha": "abc123...",
       "remote_url": "https://github.com/owner/repo"
     }
   }
   ```

## Output Files (saved to 07-deploy/)
- `DEPLOYMENT_REPORT.md` - Full deployment details
- `DEPLOYMENT.json` - Structured metadata
- `HEALTH_CHECK.md` - Verification results

## Success Criteria
- ✅ Vercel deployment succeeds
- ✅ URL is accessible
- ✅ No errors in logs
- ✅ Performance acceptable

## Next Step
Move to **Stage 08: Deployment Verification**
