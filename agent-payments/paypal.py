#!/usr/bin/env python3
"""PayPal guidance helper for the agent-payments integration."""


def paypal_info() -> str:
    return (
        "PayPal can be used to fund your Base wallet via an exchange or on-ramp. "
        "If your Coinbase Agentic Wallet on Base needs USDC or ETH, send funds to the "
        "wallet address provided by `agent-payments wallet address`. "
        "Never run send/trade/swap commands without explicit user consent. "
        "Use PayPal only as a funding source to add USDC or ETH to the Base address."
    )
