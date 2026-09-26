# QR Skill Contract

## Purpose

Generate a standards-based, traceable QR asset package for an approved HTTPS campaign destination.

## Interpreter

Confirm the campaign, payload, output location, approval status, and whether the destination has been verified independently.

## Context

Read the campaign specification, approved destination URL, campaign identifier, and QR output requirements. Never derive a production URL from branding or conversation memory alone.

## Method

1. Install the exact-pinned Segno dependency from `requirements.txt`.
2. Run `scripts/generate_qr.py` with an approved HTTPS URL.
3. Preserve the generated SVG as the protected master.
4. Preserve `payload.json` and `tests/checksums.txt` as generation receipts.
5. Do not label the package scan-verified until independent decode and physical scan evidence exists.
6. Import the protected SVG into Recraft or another layout tool without regenerating, vectorizing, cropping, or covering the QR matrix.
7. Re-scan every final composed export before delivery.

## Required outputs

- SVG master
- PNG derivative
- PDF derivative
- payload receipt
- SHA-256 checksums
- later QR scan report

## Failure boundaries

Reject the run when:

- the payload is not an absolute HTTPS URL;
- the URL contains embedded credentials;
- the installed Segno version differs from the pinned version;
- the quiet-zone border is below four modules;
- an output file is missing or empty;
- anyone requests an AI image model to recreate the matrix.

## Current verification boundary

The generator proves deterministic file creation and metadata integrity. It does not prove destination availability, RSVP behavior, physical-device scanning, final artwork scanning, or production release.
