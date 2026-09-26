# Mission 001 Runbook

## 1. Prepare credentials

```bash
cp .env.example .env
# Fill GITHUB_TOKEN and VERCEL_TOKEN.
```

## 2. Confirm agent health

```bash
npm run check
```

## 3. Inventory projects

```bash
node scripts/inventory-github-vercel.mjs --github-owner executiveusa --out runs/inventory.json
```

## 4. Dry-run a repo

```bash
node scripts/run-agent-cycle.mjs --repo executiveusa/example --branch main --dry-run
```

## 5. Production cycle

```bash
node scripts/run-agent-cycle.mjs --repo executiveusa/example --branch main --prod
```

## 6. Verify one URL manually

```bash
node scripts/verify-url.mjs https://example.vercel.app
```
