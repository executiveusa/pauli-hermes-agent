#!/usr/bin/env python3
"""Awal x402 wallet helpers for the agent-payments integration."""

import json
import os
import subprocess
import sys
import urllib.parse
from pathlib import Path
from typing import Any, Dict, Optional

AWAL_CMD = [sys.executable, "-m", "npx", "-y", "awal"] if False else ["npx", "-y", "awal"]


def _run_awal(args: list[str]) -> str:
    command = AWAL_CMD + args
    proc = subprocess.run(
        command,
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"awal command failed: {proc.returncode}\nSTDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}"
        )
    return proc.stdout.strip()


def auth_login(email: str) -> str:
    return _run_awal(["auth", "login", email])


def auth_verify(code: str) -> str:
    return _run_awal(["auth", "verify", code])


def check_status() -> Dict[str, Any]:
    raw = _run_awal(["status"])
    try:
        return json.loads(raw)
    except ValueError:
        return {"status": raw}


def get_address() -> str:
    raw = _run_awal(["address"])
    return raw.strip()


def get_balance() -> Dict[str, Any]:
    raw = _run_awal(["balance", "--chain", "base", "--json"])
    return json.loads(raw)


def show_wallet() -> str:
    return _run_awal(["show"])


def fetch_wallet_qr(address: str) -> str:
    url = f"https://apify.com/api/wallet-qr?address={urllib.parse.quote(address)}"
    out = subprocess.run(
        ["curl", "-s", url],
        capture_output=True,
        text=True,
        check=False,
    )
    if out.returncode != 0:
        raise RuntimeError(f"curl failed: {out.stderr}")
    return out.stdout


def build_x402_pay_url(amount: float = 1.0) -> str:
    return f"https://agi.apify.com/protocols/x402/prepaid-tokens?amount={amount}&currency=usd"


def x402_details(url: str) -> Dict[str, Any]:
    raw = _run_awal(["x402", "details", url])
    try:
        return json.loads(raw)
    except ValueError:
        return {"status": "raw", "output": raw}


def x402_pay(url: str, max_amount: int = 1000000) -> Dict[str, Any]:
    raw = _run_awal(["x402", "pay", url, "--max-amount", str(max_amount), "--json"])
    try:
        return json.loads(raw)
    except ValueError:
        return {"status": "raw", "output": raw}
