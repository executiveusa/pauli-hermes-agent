---
name: holo-gestures
description: Shared hand-gesture input layer for the agent fleet (VisionClaw AX-022 glasses, phone, or webcam). Use when the user wants to control things with hand gestures on camera - pinch, tap, drag, flick, two-hand stretch, peace-sign reset, point, fist, open palm - or asks what their hands are doing on a live feed. Canonical engine lives in executiveusa/VisionClaw; this skill is the Hermes-side caller.
---

# Holo Gestures (AX-022 shared capability)

Hand landmarks in, semantic gesture events out. One input layer for every agent
in the fleet: the glasses wearer pinches in mid-air, this skill classifies it,
and the consuming agent decides what the gesture means in its context.

## Source and license

Gesture rules ported from
[zubair-trabzada/holo-gestures](https://github.com/zubair-trabzada/holo-gestures)
(MIT License, (c) 2026 Zubair Trabzada / AI Workshop Studio LLC) at commit
`55626ff`. Only the gesture layer - the paywalled "Jarvis brain" and the notes
UI are excluded. Canonical implementation:
`executiveusa/VisionClaw` -> `packages/ax022-core/src/gestures/`.

## How to call it

The capability is served by the AX-022 gateway (`services/ax022-gateway` in
VisionClaw). All fleet agents use the same HTTP contract:

1. Pair a session: `POST /v1/sessions` with `{ pairingSecret, wearableId, userId, tenantId, deviceProfile }`
   -> returns a bearer `token`. (Pairing secret comes from deployment config, never from chat.)
2. Classify frames: `POST /v1/gestures` with `Authorization: Bearer <token>` and
   `{ "frames": [{ "ts": <ms>, "hands": [[21 landmarks], ...] }] }`
   -> returns `{ events, hands, receipt }`. Events include `pinch_start`,
   `pinch_end`, `tap`, `drag`, `flick`, `two_hand_stretch`, `peace`, `point`,
   `fist`, `open_palm`, `hand_lost`.
3. Engine state (pinch lifecycle, hand identity, smoothing) persists per
   session token - feed frames in order, do not create a new session per frame.

`scripts/gesture_client.py` is a stdlib-only reference client:

```bash
python3 scripts/gesture_client.py --gateway http://127.0.0.1:8790 \
  --pairing-secret "$AX022_PAIRING_SECRET" --tenant personal \
  --wearable rayban-001 --user bambu --frames frames.json
```

`frames.json` is a list of `{ "ts": ms, "hands": [...] }`. Landmarks are
MediaPipe 21-point normalized `{x, y}` sets captured on the device (MediaPipe
is Apache-2.0 and runs on-device; camera frames never need to leave it).

## Gesture -> action safety

Classification is L0 (read-tier) under the ICM policy engine. Mapping a
gesture to a consequential action (send, spend, publish, delete) must pass the
consuming agent's existing policy gate - L2 standing policy, L3 approval,
L4 hard deny. This skill never executes actions itself.
