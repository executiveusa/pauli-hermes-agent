# Workflow 01 — Bluehost / cPanel

## Goal

Safely install, migrate, restore, or repair a site on Bluehost through the browser without guessing the serving root or changing unrelated infrastructure.

## Gate 0 — ICM intake

Record:

```text
MODE:
OUTCOME:
TARGET DOMAIN:
BLUEHOST SITE/ACCOUNT:
SOURCE ARTIFACT/REPO:
EXPECTED STACK:
CONSTRAINTS:
PROOF REQUIRED:
COMMERCIAL VALUE:
AUTHORITY:
ROLLBACK:
HUMAN APPROVER:
```

If target identity or authority is unclear, inspect only.

## Gate 1 — Detect and read current docs

1. Confirm the UI is Bluehost/cPanel.
2. Read current official Bluehost docs for File Manager, Domains/document root, addon domains, and any stack-specific runtime concern.
3. Record the docs used and any discrepancy with remembered UI labels.
4. Do not mutate yet.

## Gate 2 — Prove target

Prove the exact website and serving directory at least two ways when possible:

- Bluehost Websites / Files & Access value;
- cPanel Domains document-root value;
- File Manager breadcrumb;
- known production files tied to the domain.

Record the full path.

Never default blindly to `/public_html`.

## Gate 3 — Baseline

Before changes:

- show hidden files;
- inventory the production root;
- record `index.html`, `index.php`, `.htaccess`, WordPress files, and asset directories;
- identify unrelated client material that must remain untouched;
- confirm rollback source or backup;
- note current public symptom/status.

Do not diagnose SSL, DNS, or WordPress as root cause without evidence.

## Gate 4 — Classify source

### Static / Webflow
Deploy the export/build output.

### Front-end framework
Deploy `dist/`, `build/`, `out/`, or other verified production artifact rather than raw source unless Bluehost's current official workflow builds it.

### PHP
Verify current supported PHP version/modules first.

### WordPress
Treat files + database + configuration as one unit.

### Node / SSR
Use only if the current Bluehost plan and official docs explicitly support the needed runtime. Otherwise return `BLOCKED` and recommend a supported target; do not fake a static deployment.

## Gate 5 — Stage

For file-based deployment:

```text
<verified-document-root>/_deploy_stage
```

Upload or extract only the approved artifact there.

Verify:

- entry file;
- no wrapper-folder mistake;
- required assets;
- site identity;
- file count/manifest plausibility;
- no secrets or private config in public-serving content.

For Webflow, verify `data-wf-site`, `data-wf-page`, title, expected content, CSS, JS, fonts, images, videos, and required CDNs when available.

If identity fails: `HOLD`.

## Gate 6 — Pre-promotion recheck

Immediately before promotion:

- re-open/reload production root;
- confirm target path has not changed;
- detect collisions;
- enumerate exactly which objects will move/replace;
- confirm rollback still exists.

Do not delete unrelated files for cleanliness.

## Gate 7 — Promote minimally

Move only approved production objects from staging to the proven root.

For a static Webflow-style export this often means:

```text
index.html
css/
js/
images/
fonts/
videos/
```

The homepage entry must end at the actual serving root, not one wrapper directory below it.

## Gate 8 — Public verification

Open the canonical public domain and verify:

- homepage loads and expected title/content appears;
- CSS/JS/assets load;
- fonts/images/video load as applicable;
- navigation works;
- important buttons/forms/popups/interactions work;
- responsive behavior works at viewports the tool can honestly prove;
- no fatal console errors;
- no staging ZIP, `.env`, private config, backup, or manifest is publicly exposed.

`PASS` requires public runtime proof.

## Gate 9 — Unauthorized-change audit

Explicitly report whether any of these changed:

```text
DNS
SSL
Nameservers
Document root
.htaccess
WordPress
Database
GitHub
Other Bluehost sites
Existing client files
Billing plan
Credentials
```

Any unplanned change => `HOLD`.

## Gate 10 — Cleanup

Only after public verification:

- remove workflow-created temporary staging artifacts;
- prefer reversible trash over permanent deletion unless policy says otherwise;
- never delete pre-existing customer files as cleanup.

## Reference invariant from 3X Plumbing

The successful reference restore proved the value of this exact sequence: exact document-root proof, isolated staging, site-identity verification, six-object minimal promotion, live browser checks, no DNS/SSL/root/database changes, and cleanup only after verification.

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
