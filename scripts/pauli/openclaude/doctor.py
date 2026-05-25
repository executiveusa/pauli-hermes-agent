#!/usr/bin/env python3
"""Wrapper entrypoint for the canonical Pauli OpenClaude registry doctor."""

from pauli.openclaude.dispatcher import doctor


if __name__ == "__main__":  # pragma: no cover - script entry point
    raise SystemExit(doctor())
