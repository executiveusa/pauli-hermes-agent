# Setup Guide

- Copy `.env.example` to `.env`.
- Start API: `cargo run -p foundry-api`.
- Optional local stack: `docker compose up`.
- Verify:
  - `/healthz`
  - `/readyz`
  - `/version`
  - `/deployment-manifest.json`
