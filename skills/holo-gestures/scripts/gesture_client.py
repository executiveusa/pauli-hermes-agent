#!/usr/bin/env python3
"""Stdlib-only client for the AX-022 shared holo-gestures capability.

Pairs a wearable session and classifies camera-frame hand landmarks into
gesture events via the AX-022 gateway (services/ax022-gateway in
executiveusa/VisionClaw). Part of the holo-gestures Hermes skill.

Frames file format: JSON list of {"ts": <ms epoch>, "hands": [hand, ...]}
where each hand is 21 MediaPipe-style normalized {"x", "y"} landmarks.
"""
import argparse
import json
import sys
import urllib.request


def _post(url, payload, token=None):
    req = urllib.request.Request(
        url,
        data=json.dumps(payload).encode("utf-8"),
        headers={"content-type": "application/json"}
        | ({"authorization": f"Bearer {token}"} if token else {}),
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=30) as res:
        return json.loads(res.read().decode("utf-8"))


def pair_session(gateway, pairing_secret, tenant, wearable, user, device_profile):
    body = _post(f"{gateway}/v1/sessions", {
        "pairingSecret": pairing_secret,
        "tenantId": tenant,
        "wearableId": wearable,
        "userId": user,
        "deviceProfile": device_profile,
    })
    if not body.get("ok"):
        raise SystemExit(f"pairing failed: {body}")
    return body["token"]


def classify(gateway, token, frames):
    body = _post(f"{gateway}/v1/gestures", {"frames": frames}, token=token)
    if not body.get("ok"):
        raise SystemExit(f"classify failed: {body}")
    return body


def main():
    ap = argparse.ArgumentParser(description="AX-022 holo-gestures client")
    ap.add_argument("--gateway", default="http://127.0.0.1:8790")
    ap.add_argument("--pairing-secret", required=True)
    ap.add_argument("--tenant", required=True)
    ap.add_argument("--wearable", required=True)
    ap.add_argument("--user", required=True)
    ap.add_argument("--device-profile", default="brilliant-halo")
    ap.add_argument("--frames", help="path to frames JSON; omit to pair only")
    args = ap.parse_args()

    token = pair_session(args.gateway, args.pairing_secret, args.tenant,
                         args.wearable, args.user, args.device_profile)
    print(json.dumps({"ok": True, "session": "paired", "wearable": args.wearable}))
    if args.frames:
        with open(args.frames, "r", encoding="utf-8") as fh:
            frames = json.load(fh)
        print(json.dumps(classify(args.gateway, token, frames), indent=2))


if __name__ == "__main__":
    sys.exit(main())
