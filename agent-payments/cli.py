#!/usr/bin/env python3
"""Agent-payments CLI for Apify, x402, and PayPal wallet flows."""

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

from apify import (
    discover_actors,
    inspect_actor,
    run_actor,
    make_actor_url,
    get_apify_token,
)
from awal import (
    auth_login,
    auth_verify,
    check_status,
    get_address,
    get_balance,
    show_wallet,
    x402_details,
    x402_pay,
    build_x402_pay_url,
    fetch_wallet_qr,
)
from paypal import paypal_info


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Agent-payments CLI: Apify Actors, x402 wallet payments, and PayPal funding guidance."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    auth_parser = subparsers.add_parser("auth", help="Manage Coinbase Agentic Wallet auth")
    auth_sub = auth_parser.add_subparsers(dest="subcommand", required=True)
    auth_login_parser = auth_sub.add_parser("login", help="Send login code to email")
    auth_login_parser.add_argument("email", help="Email address for awal auth login")
    auth_verify_parser = auth_sub.add_parser("verify", help="Verify the login code")
    auth_verify_parser.add_argument("code", help="Verification code from email")
    auth_sub.add_parser("status", help="Show awal authentication status")

    wallet_parser = subparsers.add_parser("wallet", help="Check wallet status and funding")
    wallet_sub = wallet_parser.add_subparsers(dest="subcommand", required=True)
    wallet_sub.add_parser("address", help="Show the wallet address")
    wallet_sub.add_parser("balance", help="Show wallet balance on Base")
    wallet_sub.add_parser("show", help="Open the wallet UI")
    wallet_sub.add_parser("qr", help="Fetch a QR code for the wallet address")

    x402_parser = subparsers.add_parser("x402", help="Pay for Apify prepaid tokens using x402")
    x402_sub = x402_parser.add_subparsers(dest="subcommand", required=True)
    x402_details_parser = x402_sub.add_parser("details", help="Show x402 payment requirements for an Apify prepaid-token URL")
    x402_details_parser.add_argument("url", help="The x402 prepaid-token details URL")
    x402_pay_parser = x402_sub.add_parser("pay", help="Pay a small USDC amount over x402")
    x402_pay_parser.add_argument("amount", type=float, default=1.0, help="USDC amount to pay")
    x402_pay_parser.add_argument("--max-amount", type=int, default=1000000, help="Safety cap in atomic USDC units")
    x402_pay_parser.add_argument("--url", default="https://agi.apify.com/protocols/x402/prepaid-tokens?amount=1&currency=usd", help="The x402 payment URL")

    apify_parser = subparsers.add_parser("apify", help="Discover and run Apify Actors")
    apify_sub = apify_parser.add_subparsers(dest="subcommand", required=True)
    apify_search = apify_sub.add_parser("discover", help="Search Apify Actors by query")
    apify_search.add_argument("query", help="Search terms for Apify Actors")
    apify_search.add_argument("--limit", type=int, default=5, help="Maximum number of actors to return")
    apify_sub.add_parser("token", help="Show the current Apify token from env")
    apify_inspect = apify_sub.add_parser("inspect", help="Fetch an Actor markdown page")
    apify_inspect.add_argument("actor_id", help="Actor id in username~name form")
    apify_run = apify_sub.add_parser("run", help="Run an Apify Actor sync and return items")
    apify_run.add_argument("actor_id", help="Actor id in username~name form")
    apify_run.add_argument("--input", help="JSON input payload for the actor")
    apify_run.add_argument("--input-file", help="Path to a JSON file containing input")

    subparsers.add_parser("paypal", help="Show PayPal funding guidance for Base wallet")

    load_env = subparsers.add_parser("load-env", help="Load environment variables from a .env or env file")
    load_env.add_argument("path", help="Path to the .env file containing API keys")

    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.command == "auth":
        if args.subcommand == "login":
            output = auth_login(args.email)
            print(output)
        elif args.subcommand == "verify":
            output = auth_verify(args.code)
            print(output)
        elif args.subcommand == "status":
            status = check_status()
            print(json.dumps(status, indent=2, ensure_ascii=False))

    elif args.command == "wallet":
        if args.subcommand == "address":
            print(get_address())
        elif args.subcommand == "balance":
            balance = get_balance()
            print(json.dumps(balance, indent=2, ensure_ascii=False))
        elif args.subcommand == "show":
            print(show_wallet())
        elif args.subcommand == "qr":
            address = get_address()
            if not address:
                print("No wallet address available. Authenticate first.")
                return 1
            qr = fetch_wallet_qr(address)
            print(qr)

    elif args.command == "x402":
        if args.subcommand == "details":
            data = x402_details(args.url)
            print(json.dumps(data, indent=2, ensure_ascii=False))
        elif args.subcommand == "pay":
            url = args.url
            if "amount=" not in url:
                url = build_x402_pay_url(args.amount)
            result = x402_pay(url, args.max_amount)
            print(json.dumps(result, indent=2, ensure_ascii=False))

    elif args.command == "apify":
        if args.subcommand == "discover":
            actors = discover_actors(args.query, args.limit)
            print(json.dumps(actors, indent=2, ensure_ascii=False))
        elif args.subcommand == "token":
            token = get_apify_token()
            print(token or "APIFY_TOKEN is not set in environment.")
        elif args.subcommand == "inspect":
            text = inspect_actor(args.actor_id)
            print(text)
        elif args.subcommand == "run":
            payload = None
            if args.input_file:
                with open(args.input_file, "r", encoding="utf-8") as f:
                    payload = json.load(f)
            elif args.input:
                payload = json.loads(args.input)
            else:
                payload = {}
            result = run_actor(args.actor_id, payload)
            print(json.dumps(result, indent=2, ensure_ascii=False))

    elif args.command == "paypal":
        print(paypal_info())

    elif args.command == "load-env":
        path = Path(args.path).expanduser()
        if not path.exists():
            print(f"Env file not found: {path}")
            return 1
        for line in path.read_text(encoding="utf-8").splitlines():
            if not line or line.strip().startswith("#"):
                continue
            if "=" not in line:
                continue
            key, value = line.split("=", 1)
            os.environ[key.strip()] = value.strip().strip('"').strip("'")
        print("Loaded environment variables from", path)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
