# Freenet Community Mesh Workflow

## Mode

Brownfield integration into Hermes. Freenet is the bounded experiment, not a replacement for the current local AI, VPS, database, or website stack.

## Outcome

Prove that two independently controlled AI nodes can exchange one approved public community resource while private organizational records remain isolated.

## Target

Initial Seattle–Everett pilot with two test nodes, expanding to five organizations only after the two-node proof passes.

## Constraints

- No production client data.
- No children, patient, donor, beneficiary, financial, immigration, legal, or private contact records.
- No autonomous live publishing.
- No open firewall or public ports without a written plan and approval.
- No claim that Freenet alpha is production-ready.
- Preserve the current owner-controlled local and cloud stack.

## Proof

### Gate 1 — deterministic simulator

Run:

```powershell
python skills/freenet-community-mesh/scripts/community_mesh.py demo `
  --workspace "$env:USERPROFILE\PauliMeshTest"
```

Then run:

```powershell
pytest skills/freenet-community-mesh/tests -q
```

Required evidence:

- `proof.json` reports `PASS`;
- two separate private stores exist;
- one resource synchronizes;
- a tampered resource is rejected;
- prohibited fields and secret patterns are rejected;
- revocation synchronizes;
- no private record appears in shared state.

### Gate 2 — machine readiness

Run `local-machine-architect` on both computers. Confirm:

- supported OS;
- free disk reserve;
- RAM and CPU suitability;
- Git, Rust, Node, and browser availability as required by the current Freenet tutorial;
- owner authorization;
- uninstall and rollback path.

### Gate 3 — local Freenet development

Use the current official Freenet dapp tutorial and example application. Freenet currently documents a contract/delegate/UI structure, local node development, TypeScript SDK support, and official AI-assisted development skills.

Start in local-only mode. Do not join the live network with client information.

Tasks:

1. Install Freenet on two non-production machines.
2. Record installer version, telemetry behavior, ports, service names, and uninstall steps.
3. Build the smallest official example.
4. Create a `CommunityResource` contract from the proven schema.
5. Add a CLI adapter for Hermes.
6. Publish synthetic public data only.
7. Test synchronization, invalid updates, revocation, restart recovery, and node outage.
8. Compare observed state with simulator expectations.

### Gate 4 — controlled two-machine trial

Use two machines under different owners.

Test matrix:

| Test | Required result |
|---|---|
| Node A publishes public resource | Node B receives exact approved fields |
| Node A stores private record | Node B cannot retrieve or infer it |
| Payload includes prohibited field | Contract or pre-publish gate rejects it |
| Payload is modified after signing | Verification rejects it |
| Node A revokes resource | Node B receives revoked state |
| One node restarts | State recovers without private leakage |
| Network unavailable | Private local AI remains usable for designated local tasks |
| Owner requests exit | Shared records and local install can be removed per manifest |

### Gate 5 — operator usability

A nontechnical operator must be able to:

- review a proposed public record;
- approve or reject publication;
- see what is shared versus private;
- revoke a record;
- stop the node;
- export local data;
- identify support contact and rollback instructions.

## Workflow states

```text
DISCOVERED
SIMULATOR_READY
SIMULATOR_VERIFIED
MACHINES_APPROVED
FREENET_LOCAL_READY
TWO_NODE_TRIAL
HUMAN_REVIEW
PILOT_APPROVED
PILOT_REJECTED
```

Never advance from `SIMULATOR_READY` to `SIMULATOR_VERIFIED` without test output. Never advance to `PILOT_APPROVED` without independent review and owner approval.

## Live adapter contract

The future adapter must expose provider-neutral commands:

```text
mesh doctor
mesh propose <resource.json>
mesh validate <resource.json>
mesh publish <resource.json>
mesh list
mesh get <resource-id>
mesh revoke <resource-id>
mesh proof
mesh rollback
```

The simulator and live Freenet adapter must produce compatible JSON so Hermes can swap transports without changing its policy layer.

## Failure handling

- Validation failure: reject before signing or publishing.
- Signature failure: quarantine record and notify owner.
- Synchronization disagreement: stop publishing, preserve evidence, compare contract state.
- Private-data detection: halt the pilot, remove shared payload where possible, rotate affected credentials, document incident.
- Unstable Freenet build: revert to simulator and existing owner-controlled infrastructure.

## Rollback

1. Stop the experimental peer or local node.
2. Export the pilot manifest and shared record list.
3. Remove only pilot contracts, delegates, services, and synthetic data.
4. Confirm private local stores remain with each owner.
5. Restore previous firewall, startup, provider, and model settings.
6. Record the rollback result.
