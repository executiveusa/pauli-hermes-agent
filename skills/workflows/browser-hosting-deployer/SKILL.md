---
name: browser-hosting-deployer
description: ICM-governed browser-agent workflow for safely installing, migrating, restoring, or repairing websites on Bluehost, Hostinger, and similar shared-hosting/control-panel environments. Detects provider and stack, reads current official provider docs before mutation, proves the exact serving target, stages before promotion, verifies site identity and public runtime, preserves unrelated client data, and requires rollback plus independent verification.
version: 1.0.0
author: Bambú / Pauli Effect
license: MIT
tags: [icm, browser-control, bluehost, hostinger, cpanel, hpanel, webflow, static-sites, php, wordpress, nodejs, deployment, migration, restore, cloneflow, rollback]
triggers:
  - deploy this site to Bluehost
  - migrate this site to Bluehost
  - restore this Bluehost site
  - deploy this site to Hostinger
  - migrate this site to Hostinger
  - install this website on shared hosting
  - browser hosting deployer
  - publish this Cloneflow export
entry_point: /browser-hosting-deployer
---

# Browser Hosting Deployer

## Purpose

Turn browser-controlled shared hosting from an ad hoc manual task into a governed, repeatable deployment workflow.

Hermes acts as interpreter, environment detector, browser operator, evidence collector, and coordinator. It must never guess the provider, document root, runtime support, or deployment state. Production mutation is allowed only inside the approved target and must be reversible.

This skill is the portable deployment half of the Cloneflow pipeline:

```text
Cloneflow
  -> extract / build / verify site bundle
  -> deployment manifest
  -> Browser Hosting Deployer
  -> provider-specific install
  -> public runtime verification
```

## Native entry point

```text
/browser-hosting-deployer <domain, source artifact/repo, requested outcome>
```

## Core invariant

**Inspect -> classify -> read current provider docs -> prove target -> baseline -> stage -> verify identity -> promote -> test public runtime -> audit -> clean up -> report.**

A successful upload is not a successful deployment. `PASS` requires public target-environment evidence.

## ICM operating model

### Interpreter

Before substantial action record:

- MODE: `brownfield` for an existing hosted site; `greenfield` for a new hosting target;
- OUTCOME: measurable result, e.g. `example.com serves the approved site bundle with HTTP 200 and required interactions working`;
- TARGET: exact provider account, website entry, domain, document root/runtime app, and source artifact;
- CONSTRAINTS: what must not change, including unrelated files/sites, DNS, SSL, nameservers, database, WordPress, billing plan, credentials, and domain ownership unless explicitly in scope;
- PROOF: exact target-path evidence, staged artifact checks, public HTTP/browser checks, asset/runtime checks, interaction checks, console errors, and independent verification;
- COMMERCIAL VALUE: restored customer site, avoided outage, migration revenue, reduced hosting cost, retained customer, or reusable Cloneflow deployment capability;
- AUTHORITY: read-only actions, bounded file actions, and actions requiring human approval;
- ROLLBACK: backup, prior file set, prior Git ref, provider restore point, or reversible move plan;
- HUMAN APPROVER: owner/operator for consequential production, DNS, billing, credential, database, or plan changes.

### Context

Load only the smallest relevant context:

- target domain and provider account identity;
- current provider UI and official documentation for the exact operation;
- source repository/build/export and deployment manifest;
- current document-root/runtime configuration;
- production root listing and hidden files where available;
- relevant backup/restore point;
- prior deployment or incident receipts for the same target;
- stack-specific runtime requirements.

Do not autoload unrelated hosting accounts, projects, repositories, or credentials.

### Method

1. Detect provider and environment.
2. Read current official provider documentation before mutation.
3. Classify stack/runtime.
4. Prove the exact serving target at least two ways when possible.
5. Baseline and preserve current state.
6. Stage the candidate deployment separately from production.
7. Verify artifact identity and runtime suitability.
8. Promote only intended objects.
9. Verify the public site and required interactions.
10. Run an unauthorized-change audit.
11. Clean temporary staging artifacts only after verification.
12. Produce the completion record.

A separate verifier should inspect the final public target. The builder/operator must not approve itself.

## Authority policy

### Automatic: READ / ANALYZE

Hermes may automatically:

- identify provider from the live UI and account context;
- inspect Websites, Domains, Files & Access, File Manager, hPanel/cPanel, runtime settings, logs, and existing files;
- show hidden files;
- read current official provider documentation;
- inspect source bundles/repositories and determine stack;
- compare production state with an approved manifest;
- calculate collisions and blast radius;
- generate a deployment, rollback, and verification plan.

### Bounded automatic file operations

Only inside the explicitly proven target and with rollback available, Hermes may:

- create an isolated staging directory;
- upload an approved ZIP/build artifact;
- extract into staging;
- verify filenames, identity markers, counts, sizes, and visible content;
- move approved deployment objects from staging to the exact document root;
- remove only temporary staging artifacts created by this workflow after public verification.

### Human approval required

Unless already explicitly authorized in the user's current instruction:

- changing DNS, nameservers, domain ownership, SSL policy, redirects, or document-root configuration;
- changing hosting plans or spending money;
- database import, migration, destructive restore, or deletion;
- WordPress install/reinstall when not part of the approved plan;
- credential creation/rotation;
- deleting pre-existing client files;
- replacing a currently working production site without a proven rollback;
- deploying executable recovery endpoints or arbitrary remote-fetch scripts;
- publishing a source build whose identity cannot be proven.

### Prohibited

- guessing that every Bluehost domain serves `/public_html`;
- guessing that every Hostinger app should use File Manager;
- copying an unbuilt Node/SSR source tree into a static web root and calling it deployed;
- treating SSL as the cause of a missing homepage without evidence;
- changing DNS/SSL/WordPress/database to troubleshoot a file-serving problem without proof;
- exposing `.env`, credentials, backups, source archives, private manifests, or secrets from a public web root;
- leaving an internet-reachable recovery webshell when provider-native file/Git/SSH mechanisms suffice;
- fabricating hashes, exact viewport tests, status codes, or provider capability;
- declaring success from upload, extraction, build, or deployment status alone.

## Provider detection and current-doc rule

The live provider and current account plan are the source of truth.

### Bluehost

Before mutation, read current official Bluehost documentation relevant to:

- File Manager;
- Domains / document root;
- addon-domain serving directories;
- `.htaccess` / directory index behavior when relevant;
- PHP or other runtime support relevant to the detected stack.

Official docs root: `https://www.bluehost.com/help`

### Hostinger

Before mutation, read current official Hostinger documentation relevant to:

- hPanel File Manager;
- Git deployment;
- PHP/static hosting;
- Node.js Web App deployment when relevant;
- domain and document-root behavior.

Official docs root: `https://www.hostinger.com/support`

If the current docs or account UI conflict with remembered instructions, follow the current docs/UI and record the discrepancy.

## Stack classifier

### Static / Webflow export

Signals:

- `index.html`;
- `.html` pages;
- `css/`, `js/`, `images/`, `fonts/`, `videos/`;
- Webflow markers such as `data-wf-site` and `data-wf-page`;
- no required server runtime.

Deploy the built/exported files to the proven serving root.

For Webflow exports, verify when available:

- page title;
- `data-wf-site`;
- `data-wf-page`;
- expected branded text;
- Webflow CSS and JS;
- required images/fonts/videos;
- externally hosted dependencies that remain necessary.

### Front-end framework build

Signals:

- source project contains `package.json`, but production output is `dist/`, `build/`, `out/`, or equivalent.

Deploy the production build output, not raw source, unless the provider's official build service is intentionally used.

### PHP

Signals:

- `index.php`;
- Composer/PHP files;
- no required long-running Node process.

Verify supported PHP version/modules before promotion.

### WordPress

Signals:

- `wp-admin/`, `wp-content/`, `wp-includes/`, `wp-config.php`;
- database dependency.

Treat files + database + configuration as one deployment unit. Files-only restoration is not completion.

### Node.js / SSR / server application

Signals:

- `package.json` with server/start scripts;
- Next.js SSR, Express, Nest, or another long-running runtime.

Verify provider and plan support. Use the provider's official Node/Web App workflow. Do not force the app into a static document root.

### Unsupported runtime

Stop with `BLOCKED` and explain the incompatibility. Do not improvise a hosting architecture that the provider/plan does not support.

## Target proof contract

Before upload or extraction, record:

```text
Provider:
Account/site:
Domain:
Serving mode: static | PHP | WordPress | Git | Node Web App | other
Document root or runtime app:
Target proof #1:
Target proof #2:
Current homepage/index state:
Existing files to preserve:
Rollback source:
```

Use two independent proofs when possible, for example:

- provider Websites page + File Manager breadcrumb;
- Domains table + Files & Access document-root value;
- Node Web App project identity + domain binding;
- current known production files + domain configuration.

## Staging contract

For file-based deployments, prefer an isolated directory such as:

```text
<verified-document-root>/_deploy_stage
```

Validate before promotion:

- correct entry file exists;
- no accidental wrapper folder;
- required CSS/JS/assets exist;
- file count/manifest is plausible;
- expected site identity matches;
- bundle belongs to the requested project;
- no secrets/private source artifacts are exposed;
- runtime matches provider capability.

If validation fails: `HOLD`. Do not promote.

## Promotion contract

Re-check production immediately before promotion.

Move only approved production artifacts, e.g. for a static export:

```text
index.html
css/
js/
images/
fonts/
videos/
```

Preserve unrelated files. Detect collisions. Never move a wrapper directory one level below the serving root unless that subdirectory is intentionally the configured root.

## Public proof contract

After promotion verify the actual public domain, not just File Manager:

- homepage resolves at the canonical URL;
- expected content/title is visible;
- CSS and JS load;
- critical images/fonts/video load;
- required CDN dependencies load;
- no fatal console errors;
- navigation works;
- required forms/buttons/popups/interactions work as applicable;
- responsive behavior is checked honestly at the viewport sizes the browser tool can actually prove;
- no sensitive staging/archive/config file is publicly accessible.

If a check cannot be performed, mark it `NOT_RUN` or `UNVERIFIED`; never fabricate proof.

## Unauthorized-change audit

Report explicitly whether the workflow changed:

```text
DNS
SSL
Nameservers
Document root
.htaccess
WordPress
Database
GitHub
Other hosting sites
Existing client files
Billing plan
Credentials
```

Unexpected changes force `HOLD` until reviewed.

## Lessons encoded from the 3X Plumbing restore

The reference Bluehost restore succeeded because the browser agent:

- proved the exact site and document root instead of assuming `/public_html`;
- found that the production root had no `index.html`, CSS, JS, images, fonts, or videos;
- preserved unrelated existing client files;
- staged the supplied Webflow ZIP in an isolated directory;
- verified Webflow site/page IDs, title, CSS, fonts, images, videos, and JS before promotion;
- promoted only the intended six objects: `index.html`, `css/`, `fonts/`, `images/`, `js/`, `videos/`;
- did not change DNS, SSL, nameservers, document root, `.htaccess`, WordPress, database, GitHub, Vercel, Webflow, or other Bluehost sites;
- tested the live homepage, assets, responsive behavior, navigation, popups, IX2 animation, and ticker;
- refused an unnecessary token-authenticated remote-fetch PHP recovery endpoint because provider-native static file operations were sufficient;
- reported that SHA-256 and exact viewport checks were unavailable where the UI/tool could not prove them instead of fabricating evidence;
- removed staging only after the public site passed.

These are invariants, not one-off instructions for one domain.

## Cloneflow handoff contract

Cloneflow should provide a deployment bundle plus a manifest containing at least:

```json
{
  "project": "example",
  "source": "webflow-export",
  "runtime": "static",
  "entry": "index.html",
  "expected": {
    "domain": "example.com",
    "title": "Example",
    "site_id": "optional-webflow-site-id",
    "page_id": "optional-webflow-page-id",
    "required_paths": ["index.html", "css", "js", "images"]
  }
}
```

Cloneflow proves bundle fidelity. This skill proves safe installation and public runtime.

## Workflow router

| Request | Workflow |
|---|---|
| Bluehost, cPanel, addon-domain, static/PHP/WordPress file install or restore | `workflows/01-bluehost-cpanel.md` |
| Hostinger hPanel, static/PHP/Git/Node Web App install or restore | `workflows/02-hostinger-hpanel.md` |
| Unknown provider | Run detection + current-doc discovery first; do not mutate until provider path is proven |

## Required evidence bundle

Preserve in the approved project ICM workspace:

- intake contract;
- provider/runtime detection;
- current-doc references used;
- source artifact identity/manifest;
- exact target proof;
- baseline production listing;
- rollback source;
- staging verification;
- collision report;
- authorization receipt for consequential actions;
- promotion receipt;
- public browser/runtime checks;
- unauthorized-change audit;
- cleanup receipt;
- independent verifier result;
- unresolved risks.

## Completion record

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

Status vocabulary:

- `PASS` — verified in the real public target environment;
- `HOLD` — evidence or approval gate failed;
- `BLOCKED` — provider/runtime/tooling incompatibility prevents safe execution;
- `NOT_RUN` — requested check/action was not executed;
- `UNVERIFIED` — action may have occurred but proof is insufficient.
