# Workflow 02 — Hostinger / hPanel

## Goal

Safely install, migrate, restore, or repair a site on Hostinger by choosing the provider-native path that matches the detected runtime: static/PHP files, Git deployment, WordPress migration, or Node.js Web App.

## Gate 0 — ICM intake

Record:

```text
MODE:
OUTCOME:
TARGET DOMAIN:
HOSTINGER SITE/ACCOUNT:
SOURCE ARTIFACT/REPO:
EXPECTED STACK:
CONSTRAINTS:
PROOF REQUIRED:
COMMERCIAL VALUE:
AUTHORITY:
ROLLBACK:
HUMAN APPROVER:
```

## Gate 1 — Detect and read current docs

1. Confirm the UI is Hostinger hPanel.
2. Read current official Hostinger docs for the exact operation:
   - File Manager/static/PHP;
   - Git deployment;
   - Node.js Web App deployment;
   - WordPress migration/restore if relevant;
   - domain/document-root behavior.
3. Record the docs used and the runtime path chosen.
4. Do not mutate until the provider-native path is proven.

## Gate 2 — Classify runtime and choose path

### Static / Webflow / front-end build output
Use File Manager or Hostinger's supported Git/static deployment path.

### PHP
Use File Manager/Git and verify supported PHP version/modules.

### WordPress
Treat files, database, configuration, and domain binding as one migration unit.

### Node.js / SSR / server app
Use Hostinger's official Node.js Web App workflow when the current plan supports it. Verify:

- repository or uploaded artifact;
- branch/ref;
- install/build command;
- output/entry point;
- runtime version;
- environment-variable configuration without exposing values;
- domain binding.

Do not copy raw Node source into `public_html` and call it deployed.

### Unsupported plan/runtime
Return `BLOCKED`. Do not force the stack into an incompatible hosting mode.

## Gate 3 — Prove target

Use at least two proofs when possible:

- hPanel Websites entry + domain;
- File Manager path/document root;
- Git deployment target directory;
- Node Web App project identity + domain binding;
- known production files/app state.

Record exact serving mode and target.

## Gate 4 — Baseline and rollback

Before mutation:

- inventory current files/app config;
- identify hidden files and existing entry points;
- preserve unrelated client content;
- confirm backup/restore point or reversible prior deployment;
- record current public behavior;
- identify database dependencies where applicable.

Do not change DNS, SSL, nameservers, billing plan, database, or ownership unless explicitly in scope.

## Gate 5A — File/Git staging path

For static/PHP file-based deployment, use isolated staging where practical:

```text
<verified-document-root>/_deploy_stage
```

Verify entry file, identity, assets, manifest, no wrapper-folder error, and absence of secrets before promotion.

For Git deployment, verify repository, branch/ref, target directory, and build/output behavior before triggering deployment.

## Gate 5B — Node Web App path

Before launch/deploy verify:

- app belongs to the requested repository/project;
- install command is correct;
- build command is correct;
- start/entry command is correct;
- runtime version is supported;
- required environment variable names are known, but secret values are not printed;
- output/port expectations match Hostinger's current Node Web App model;
- rollback target exists.

A successful build is not production proof.

## Gate 6 — Promote/deploy

### Static/PHP
Promote only approved files to the proven root.

### Git
Deploy only the approved branch/ref to the proven target.

### Node Web App
Trigger the provider-native deploy for the approved app/ref/config.

### WordPress
Execute only the approved files/database/config steps; verify consistency before switching traffic.

## Gate 7 — Public verification

Open the canonical domain and verify:

- homepage/application resolves;
- expected title/content is present;
- CSS/JS/assets load;
- server routes/API-dependent views work when applicable;
- required interactions work;
- forms and login flows are checked if in scope;
- no fatal console/runtime errors;
- no staging archive/private config/backup is publicly exposed;
- responsive behavior is checked at viewports the browser tool can prove.

For Node apps, include app/runtime health evidence, not only front-end rendering.

## Gate 8 — Unauthorized-change audit

Report explicitly whether these changed:

```text
DNS
SSL
Nameservers
Document root / target directory
Runtime/app settings
WordPress
Database
Git branch/ref
Other Hostinger sites
Existing client files
Billing plan
Credentials
```

Unexpected change => `HOLD`.

## Gate 9 — Cleanup

After public verification only:

- remove workflow-created staging/temp artifacts;
- keep rollback source according to policy;
- do not delete unrelated customer data.

## Completion

Return:

```text
DECISION
CHANGES
PROOF
STATUS
COMMERCIAL IMPACT
RISKS
ROLLBACK
NEXT
HUMAN APPROVAL
```
