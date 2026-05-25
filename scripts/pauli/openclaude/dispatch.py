#!/usr/bin/env python3
"""Wrapper entrypoint for the canonical Pauli OpenClaude dispatcher."""

from pauli.openclaude.dispatcher import main


if __name__ == "__main__":  # pragma: no cover - script entry point
    raise SystemExit(main())
