---
name: campaign-factory
description: Turn one approved campaign specification into a verified landing page, functional QR package, branded campaign assets, and a reproducible client handoff.
version: 0.1.0
author: Bambú / Pauli Effect
license: MIT
tags: [campaigns, branding, qr, landing-pages, social-media, print, icm]
platforms: [linux, macos, windows]
triggers:
  - build a campaign kit
  - create a QR campaign
  - make event flyers and social assets
  - generate a campaign landing page
  - package a nonprofit campaign
  - campaign factory
---

# Campaign Factory

## Purpose

Transform one approved campaign specification into a complete, traceable campaign package without treating AI-generated artwork, a deployment request, or an unverified QR image as finished work.

This is a workflow skill, not a monolithic application. Prove it through real campaigns before wrapping it in a micro-app.

## Operating Model

Use the repository's Interpretable Context Methodology (ICM):

- **Interpreter:** identify the campaign type, objective, audience, constraints, required approvals, and evidence.
- **Context:** load the organization brand system, approved copy, event facts, destination URL, partner rules, and prior campaign evidence.
- **Method:** execute the numbered campaign stages, write artifacts to the filesystem, stop at approval gates, and preserve receipts.

The filesystem is the coordinator. Chat history is not the source of truth.

## Mandatory Principles

1. One campaign specification is the source of truth.
2. Verify the destination before generating the production QR.
3. Generate the QR with deterministic QR software; do not ask an image model to invent the QR matrix.
4. Keep SVG as the master QR asset. Derive PNG and PDF exports from the verified master.
5. Never modify finder patterns, timing patterns, alignment patterns, or required quiet zones for aesthetics.
6. AI may design around a verified QR, but the final composed asset must be rescanned.
7. Approved client language outranks invented campaign copy.
8. Separate observation, proposal, approval, production, and verification.
9. Do not claim deployed, scan-tested, print-ready, or client-ready without evidence.
10. Preserve source files, editable masters, ownership, rollback instructions, and campaign metadata.

## Required Inputs

At minimum:

- organization name
- campaign name
- campaign owner
- measurable immediate outcome
- primary call to action
- audience or audience-routing model
- approved destination URL
- approved campaign facts and copy
- brand assets or explicit fallback rules
- requested output formats
- deadline
- constraints and forbidden changes
- human approver

Missing information must be marked `UNKNOWN`; never silently invent dates, addresses, partner names, sponsors, legal claims, or production URLs.

## ICM Stage Workflow

### 00 — Intake

Create `campaign-spec.yaml` from the client request and existing approved sources. Record assumptions separately.

### 01 — Source Audit

Inspect brand files, approved copy, existing flyers, website, RSVP flow, event facts, partner marks, and prior campaign assets. Produce `SOURCE_AUDIT.md` with conflicts and missing evidence.

### 02 — Message Lock

Define one primary message, one CTA, supporting messages, tone, prohibited framing, and audience-routing choices. Human approval is required before production copy is treated as locked.

### 03 — Destination

Build or verify the landing page and form. Confirm ownership, HTTPS, expected content, mobile behavior, privacy language, and rollback. A URL is not approved merely because it resolves.

### 04 — QR Engineering

Generate the production QR from the approved destination. Store the exact payload, error-correction choice, generator/version, output checksum, and timestamp. Export SVG master plus PNG and PDF derivatives.

### 05 — QR Verification

Test the bare QR and each branded lockup. Record devices or scanner libraries used, test sizes, contrast conditions, failures, and final result. Failed variants are rejected, not delivered.

### 06 — Campaign Art Direction

Apply the approved permanent campaign language: typography, colors, photography/illustration direction, layout rules, CTA hierarchy, logo rules, accessibility, and QR placement rules. Produce restrained variants only when they serve a defined channel or audience.

### 07 — Asset Production

Generate requested print, social, profile, banner, presentation, email, video-thumbnail, and partner-handoff assets from the same specification and design system.

### 08 — Channel QA

Verify dimensions, safe areas, text legibility, cropping, bleed, color mode, file format, accessibility, logo integrity, and QR scanning after final composition.

### 09 — Package and Handoff

Produce an organized client package with masters, exports, usage guide, campaign metadata, known limitations, and rollback instructions. Do not include secrets or private user data.

### 10 — Measurement Readiness

Record campaign identifiers and intended metrics such as family interest, volunteers, sponsors, partners, RSVP completion, and scan traffic. Metrics may be deferred, but the data model must not erase those categories.

## Quality Gates

A campaign cannot advance to handoff until:

- campaign facts match approved sources;
- destination ownership and behavior are verified;
- RSVP or target action is functional when in scope;
- QR payload is recorded and matches the approved URL;
- SVG master exists;
- final QR variants pass scan tests;
- required quiet zone and contrast are preserved;
- dimensions and safe areas are verified;
- editable masters and exports are organized;
- no sponsors or endorsements are implied without approval;
- client ownership, rollback, and handoff notes are present.

## Professional QR Package

Minimum deliverables:

```text
qr/
├── master/
│   ├── campaign-qr.svg
│   ├── campaign-qr.png
│   ├── campaign-qr.pdf
│   └── payload.json
├── lockups/
│   ├── qr-only/
│   ├── qr-logo/
│   ├── scan-to-rsvp/
│   ├── light-background/
│   ├── dark-background/
│   └── one-color/
├── tests/
│   ├── QR_TEST_REPORT.md
│   └── checksums.txt
└── QR_USAGE_GUIDE.md
```

EPS is optional and should be generated only when a printer requires it. Never use a raster PNG as the sole master.

## Campaign Handoff Package

When requested, support:

- letter flyer
- tabloid poster
- square social post
- portrait feed post
- vertical story and reel cover
- Facebook page cover
- Instagram/Facebook profile-safe assets
- LinkedIn organization assets
- YouTube thumbnail and channel banner
- email header
- presentation cover and 16:9 slide
- table sign, sticker, badge, and partner insert
- editable source files
- brand and QR usage guide

Platform dimensions must be checked at execution time; do not rely indefinitely on dimensions embedded in this skill.

## Tool Boundaries

- Use deterministic QR libraries or established QR generators for the matrix.
- Use Recraft or another design system for vector artwork, layout exploration, icons, and surrounding campaign graphics.
- Import the verified QR SVG into the design tool as a protected asset.
- Do not vectorize a screenshot of a QR code when the original SVG can be generated.
- Re-scan every final export after composition.

## Prototype Boundary

The first prototype should prove this vertical slice:

```text
approved landing page
  -> working target action
  -> verified production URL
  -> QR SVG master
  -> branded QR lockup
  -> scan-test receipt
  -> organized client ZIP
```

Do not build a dashboard or micro-app until this workflow has been completed repeatedly and the stable inputs, outputs, failure modes, and approvals are known.

## Completion Report

End substantial runs with:

- DECISION
- CHANGES
- PROOF
- STATUS
- COMMERCIAL IMPACT
- RISKS
- ROLLBACK
- NEXT
- HUMAN APPROVAL

## Related Skills

Load relevant existing skills rather than duplicating their instructions:

- `website-design` for art direction and website quality
- `interactive-artifact` for client-facing interactive prototypes
- deployment skills for the chosen host
- image-generation or vector-design tools for artwork around the QR
- research skills when current platform specifications must be verified
