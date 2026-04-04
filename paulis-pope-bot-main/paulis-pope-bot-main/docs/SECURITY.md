# Security

## Security Measures

thepopebot includes several security measures by default:

- **API key authentication** — All external `/api` routes require a valid `x-api-key` header. Keys are SHA-256 hashed in the database.
- **Webhook secret validation** — Telegram and GitHub webhook endpoints validate shared secrets. If a secret is not configured, the endpoint rejects all requests (fail-closed).
- **Session encryption** — Web sessions use JWT encrypted with `AUTH_SECRET`, stored in httpOnly cookies.
- **Secret filtering in Docker agent** — The `env-sanitizer` extension filters `AGENT_*` secrets from the LLM's bash subprocess, preventing the agent from accessing protected credentials.
- **Auto-merge path restrictions** — The `auto-merge.yml` workflow only merges PRs where all changed files fall within `ALLOWED_PATHS` (default: `/logs`). Changes outside allowed paths require manual review.
- **Server Actions with session checks** — All browser-to-server mutations use Next.js Server Actions with `requireAuth()`, which validates the session cookie before executing.

## Disclaimer

We do our best to follow security best practices, but **all software carries risk**. thepopebot is provided as-is, without warranties of any kind. You are responsible for:

- Securing your own infrastructure (server, network, DNS)
- Managing your API keys and secrets
- Reviewing agent-generated pull requests before merging outside `/logs`
- Monitoring your agent's activity and resource usage

For a detailed list of known security findings and their status, see [SECURITY_TODOS.md](SECURITY_TODOS.md).

## Running on a Local Machine

When you run `npm run dev` and expose your machine to the internet via ngrok, Cloudflare Tunnel, or port forwarding, you are making your development server publicly accessible. This is useful for testing but carries real risks.

### What's exposed

When your tunnel is active, the following endpoints are reachable from the internet:

| Endpoint | Purpose | Auth |
|----------|---------|------|
| `/api/create-job` | Creates GitHub branches and triggers Docker agent jobs | API key required |
| `/api/telegram/webhook` | Accepts incoming Telegram updates | Webhook secret required |
| `/api/github/webhook` | Accepts GitHub Actions notifications | Webhook secret required |
| `/api/ping` | Health check | None (public) |
| `/login` | Authentication page | None (public) |
| `/stream/chat` | Chat streaming endpoint | Session cookie |
| `/` (all other routes) | Web chat interface | Session cookie via middleware |

### Risks

- **No rate limiting** — There is no rate limiting on any endpoint. A determined attacker could spam job creation (burning GitHub Actions minutes and LLM API credits) or flood the login page with brute-force attempts.
- **Local filesystem access** — The Next.js dev server runs with your user permissions. It has access to your local filesystem through the project directory and any paths that `process.cwd()` can reach.
- **Local network exposure** — `npm run dev` binds to `0.0.0.0` by default. Other devices on your local network (or anyone on the same Wi-Fi) can reach the dev server directly without going through the tunnel.
- **No TLS on the dev server** — `npm run dev` serves plain HTTP. On your local network, API keys, session cookies, and webhook payloads are transmitted in cleartext. The tunnel itself provides TLS to the public internet, but the local hop between the tunnel agent and your dev server is unencrypted.
- **Persistent exposure** — Tunnels left running keep all endpoints accessible even when you're not actively developing. If you step away or close your editor but leave the tunnel up, the endpoints remain live.

### Recommendations

- **Always set webhook secrets** — Configure `TELEGRAM_WEBHOOK_SECRET` and `GH_WEBHOOK_SECRET` in your `.env`, even for local development. Without them, webhook endpoints reject all requests by default, but setting explicit secrets adds an extra layer of validation.
- **Always set API keys** — Generate an API key through the web UI before exposing your server. Without a valid key, `/api/create-job` requests are rejected.
- **Stop tunnels when not in use** — Close ngrok or your tunnel when you're done developing. Don't leave endpoints exposed overnight.
- **Restrict Telegram to your chat** — Set `TELEGRAM_CHAT_ID` in your `.env` to your personal chat ID. This ensures the bot only responds to messages from your chat, ignoring messages from anyone else who discovers the bot.
- **Use docker-compose with TLS for production** — For anything beyond local testing, deploy with `docker compose up` and enable Let's Encrypt TLS. See the [Production Deployment](../README.md#production-deployment) section in the README.
- **Review auto-merge settings** — Keep `ALLOWED_PATHS` restrictive (default `/logs`). Only widen it after reviewing what your agent might change.
