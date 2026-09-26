---
name: freenet-community-mesh
description: Governed workflow for prototyping owner-controlled local AI nodes that share deliberately public community resources through a Freenet-compatible contract boundary while keeping private organizational data local.
version: 0.1.0
author: Bambú / Pauli Effect
---

# Freenet Community Mesh

## Purpose

Use this skill when building or testing a small sovereign AI network in which each organization retains private local data and selectively publishes non-sensitive shared resources.

This is a bounded experiment. Freenet is under active development and its own quickstart warns not to use alpha builds for sensitive data. Do not place production client records, credentials, donor data, children's data, medical data, legal files, or private organizational memory on the shared layer.

## Architecture

```text
Private node
  local documents, embeddings, models, logs, credentials
        |
        | explicit publication gate
        v
CommunityResource contract
  public service, location, language, availability, contact method
        |
        v
Freenet-compatible shared transport
```

Freenet applications separate shared state in contracts, private local state in delegates, and browser or CLI interaction in the UI. Hermes should preserve the same boundary even when running the deterministic simulator before a live Freenet adapter exists.

## Trigger phrases

- build a Freenet prototype
- create a community mesh
- connect local AI nodes
- sovereign AI cooperative
- Seattle community resource network
- test private versus shared state

## Required workflow

1. **Intake**
   - Identify the two or more organizations participating.
   - Define one shared problem with measurable value.
   - List data that may be public and data that must remain private.
   - Record an owner for each node.

2. **Read-only machine audit**
   - Invoke `local-machine-architect` on each machine.
   - Confirm disk reserve, RAM, operating system, and installed development tools.
   - Do not install Freenet or models until the audit is reviewed.

3. **Contract boundary**
   - Use only the `CommunityResource` schema in the prototype.
   - Reject unknown fields.
   - Reject private-field names and private payload patterns.
   - Require organization identity, timestamp, and signature metadata.

4. **Deterministic simulator proof**
   - Run two isolated node directories.
   - Add private local records to both nodes.
   - Publish one approved public resource from Node A.
   - Synchronize shared resources to Node B.
   - Verify Node B cannot read Node A private records.
   - Revoke the shared resource and verify the revocation synchronizes.

5. **Human review**
   - Review the exact shared payload.
   - Confirm no private fields or sensitive content escaped.
   - Record PASS or HOLD.

6. **Live Freenet trial**
   - Only after simulator PASS.
   - Install Freenet on two non-production machines.
   - Use local mode first.
   - Build from the official Freenet tutorial or `freenet-ping` example.
   - Replace simulator transport with a contract adapter while preserving the schema and tests.
   - Do not use sensitive data.

7. **Commercial proof**
   - Measure retrieval time, setup time, failure recovery, operator usability, and data-isolation results.
   - Do not claim decentralization, privacy, outage resilience, or interoperability until the corresponding test passes.

## CommunityResource schema

```json
{
  "resource_id": "uuid-or-stable-id",
  "organization_id": "public-organization-id",
  "category": "food|transport|youth|translation|health-navigation|housing-navigation|jobs|other",
  "service": "public description",
  "location": "public service area",
  "languages": ["en", "es"],
  "availability": "public availability statement",
  "contact_method": "public contact method",
  "updated_at": "ISO-8601 timestamp",
  "status": "active|revoked",
  "signature": "prototype signature or live delegate signature"
}
```

## Prohibited shared fields

The shared layer must reject fields or content representing:

- client, patient, donor, student, child, or beneficiary records;
- private names, addresses, phone numbers, emails, case notes, or identifiers;
- credentials, tokens, API keys, private keys, passwords, cookies, or sessions;
- private documents, embeddings, transcripts, internal messages, or agent memory;
- financial account data, medical data, immigration status, or legal strategy.

## Agent authority

Hermes may:

- generate schemas, tests, manifests, local simulator data, and install plans;
- run read-only audits;
- create proposed public resource records;
- run deterministic synchronization and leakage tests.

Hermes must obtain human approval before:

- installing Freenet;
- exposing ports or changing firewall rules;
- publishing any resource to a live peer-to-peer network;
- connecting a client machine;
- creating persistent services;
- using any paid model or infrastructure;
- changing a production workflow.

Hermes must never:

- publish private organizational records;
- infer that data is public merely because it is found on a local machine;
- silently enable telemetry or network exposure;
- claim that the live Freenet prototype is production-ready.

## Acceptance criteria

The simulator passes only when:

- two nodes maintain separate private stores;
- an approved public resource synchronizes;
- private records never appear in shared state or another node's files;
- prohibited fields are rejected;
- tampered records are rejected;
- revocation synchronizes;
- all tests pass;
- original input files remain unchanged.

The live trial passes only when the same contract tests pass on two non-production Freenet peers and a human confirms usability.

## Rollback

- Stop the live Freenet peers.
- Remove only the experimental contract, delegate, and generated test data listed in the manifest.
- Restore the previous network and model configuration.
- Retain private local data under the original owner.
- Delete shared experimental records when required by the pilot agreement.
