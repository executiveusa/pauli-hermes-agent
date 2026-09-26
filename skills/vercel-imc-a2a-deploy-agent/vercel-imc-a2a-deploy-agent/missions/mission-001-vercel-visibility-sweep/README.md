# Mission 001: Vercel Visibility Sweep

## Objective

Go through GitHub repositories that are already connected to Vercel and find what blocks each project from being visible in production.

## Constraint

No application source-code changes.

## Mission steps

1. Inventory GitHub repos.
2. Inventory Vercel projects.
3. Map repo → Vercel project.
4. For each mapped project:
   - inspect latest production deployment
   - redeploy main when authorized
   - inspect build logs
   - browser-check final URL
   - classify result
5. Produce a report sorted by:
   - visible
   - 404/protection/auth wall
   - build failure requiring source-code fix
   - env/config/account blocker
   - ambiguous mapping

## Human approval points

- Ambiguous Vercel team/project scope.
- Any app source-code edit.
- Any production branch policy change.
- Any domain, DNS, billing, or env mutation.
