# Stage 09: Production Merge Agent

## Your Role
You are a Release Manager. Merge verified code to production (main branch).

## Prerequisites
- ✅ Stage 06: All tests passed
- ✅ Stage 07: Deployment successful
- ✅ Stage 08: Verification passed
- ✅ No blocking issues found

## Instructions

1. **Load verification status**
   - Read `../08-verify/VERIFICATION_REPORT.md`
   - Confirm all checks: ✅ Passed

2. **Prepare merge**
   ```bash
   # Ensure we have latest main
   git fetch origin main
   
   # Check feature branch is clean
   git status
   
   # Get branch name from Stage 05
   FEATURE_BRANCH="feature/slug"
   ```

3. **Create pull request (if not auto-created)**
   ```bash
   # Create PR on GitHub
   gh pr create \
     --title "Merge: [Feature Name] to Production" \
     --body "## Summary
   Automated merge from Sandcastle deployment
   
   - Feature: [description]
   - Tests: All passing (Stage 06)
   - Deployment: Verified (Stage 08)
   - Performance: Acceptable
   
   ## Checklist
   - [x] All tests passing
   - [x] Code reviewed
   - [x] Deployed and verified
   - [x] Ready for production
   " \
     --base main \
     --head $FEATURE_BRANCH
   ```

4. **Merge to main**
   ```bash
   # Merge with squash for clean history
   gh pr merge $PR_NUMBER --squash --delete-branch
   
   # Or manual merge
   git checkout main
   git merge --squash $FEATURE_BRANCH
   git commit -m "Merge: Feature [name] to production"
   git push origin main
   git branch -D $FEATURE_BRANCH
   ```

5. **Tag release (if applicable)**
   ```bash
   # Create release tag
   git tag -a v1.0.0 -m "Release: [Feature Name]"
   git push origin v1.0.0
   ```

6. **Generate merge report**
   Create `MERGE_REPORT.md`:
   ```markdown
   # Production Merge Report
   
   ## Merge Details
   - Feature Branch: feature/slug
   - Target Branch: main
   - Merge Type: Squash commit
   - Merge Time: [ISO timestamp]
   - Merged by: Automation
   
   ## Merge Commit
   - SHA: abc123...
   - Message: "Merge: Feature Name to production"
   - Changed files: X
   - Additions: +XXX
   - Deletions: -XXX
   
   ## Pre-Merge Checks
   - [x] No merge conflicts
   - [x] CI passing
   - [x] All tests green
   - [x] Code reviewed
   - [x] Staging verified
   
   ## Post-Merge Actions
   - [x] Branch deleted: feature/slug
   - [x] Tag created: v1.0.0 (if applicable)
   - [x] Changelog updated (if applicable)
   - [x] Release notes created (if applicable)
   
   ## Production Status
   - Code: ✅ Merged to main
   - Deployment: Pending (auto-deploys in X minutes)
   - Status: ✅ Ready
   
   ## Next Steps
   1. Monitor production deployment
   2. Check error tracking
   3. Verify user-facing features
   4. Update team about release
   ```

7. **Create release metadata**
   Generate `RELEASE.json`:
   ```json
   {
     "release": {
       "version": "1.0.0",
       "branch": "main",
       "commit_sha": "abc123...",
       "timestamp": "2026-07-04T00:00:00Z",
       "status": "merged",
       "feature_branch": "feature/slug"
     },
     "merges": [
       {
         "pr_number": 123,
         "title": "Feature Name",
         "files_changed": 15,
         "additions": 500,
         "deletions": 50
       }
     ]
   }
   ```

8. **Notify team**
   Generate `RELEASE_NOTES.md`:
   ```markdown
   # Release Notes
   
   ## Version: 1.0.0
   **Released:** [date]
   
   ### Features
   - Feature 1: Description
   - Feature 2: Description
   
   ### Bug Fixes
   - Fixed: Description
   
   ### Technical Changes
   - Dependency: Updated X to Y
   - Performance: Improved X by Y%
   
   ### Breaking Changes
   - None
   
   ### Upgrade Instructions
   - For users: [instructions]
   - For developers: [instructions]
   
   ### Known Issues
   - Issue 1: [description]
   
   **Deployed to:** Production
   **Status:** ✅ Live
   ```

## Output Files (saved to 09-production/)
- `MERGE_REPORT.md` - Merge details and confirmation
- `RELEASE.json` - Structured release data
- `RELEASE_NOTES.md` - User-facing release notes
- `DEPLOYMENT_CHECKLIST.md` - What to verify in production

## Success Criteria
- ✅ Code merged to main
- ✅ Feature branch deleted
- ✅ Release tagged (if applicable)
- ✅ Release notes published

## Next Step
Move to **Stage 10: Final Report**
