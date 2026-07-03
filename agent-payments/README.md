# agent-payments

A lightweight CLI integration for Apify Actor discovery and execution using Coinbase Agentic Wallet (`awal`) and x402 prepaid token purchases.

## Features

- Authenticate with `npx awal auth login <email>` and `npx awal auth verify <code>`
- Check Base wallet balance and address
- Fetch a scannable QR code for funding
- Buy Apify prepaid tokens via x402 using `awal`
- Discover Apify Actors by search term
- Inspect Actor markdown pages
- Run Apify Actors sync with a spend-capped prepaid token
- PayPal guidance for Base wallet funding

## Quick start

```bash
python agent-payments/cli.py auth login you@example.com
python agent-payments/cli.py auth verify 123456
python agent-payments/cli.py wallet balance
python agent-payments/cli.py x402 details https://agi.apify.com/protocols/x402/prepaid-tokens?amount=1&currency=usd
python agent-payments/cli.py x402 pay --amount 1
python agent-payments/cli.py apify discover "web search news" --limit 5
```

## Requirements

- `npx`
- `curl`
- `APIFY_TOKEN` in env for running Actors
- `HOSTINGER_API_TOKEN` not required for this integration
