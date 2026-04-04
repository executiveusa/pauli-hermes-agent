# Production Deployment

## Local Development

```bash
npm run dev    # Next.js dev server
```

## Production (Docker Compose)

```bash
npx thepopebot init   # Scaffold project
npm run setup          # Configure .env, GitHub secrets, Telegram
npm run build          # Next.js build → generates .next/
docker-compose up      # Start Traefik + event handler + runner
```

## Event Handler Docker Image

The event handler Dockerfile (`templates/docker/event-handler/Dockerfile`) is **not a self-contained application image**. It only provides the Node.js runtime, system dependencies (git, gh, python3, build tools), PM2, and pre-installed `node_modules`. It does **not** contain the Next.js app code and does **not** run `next build`.

### How the two volume mounts work together

```yaml
volumes:
  - .:/app              # bind mount: host project → /app
  - /app/node_modules   # anonymous volume: preserves container's node_modules
```

The bind mount (`.:/app`) overlays the entire `/app` directory with the host's project files — app pages, config, `.next/`, `.env`, everything. This **would** also clobber the container's `/app/node_modules` with the host's macOS-compiled node_modules. But the anonymous volume (`/app/node_modules`) shields that specific path from the bind mount. Docker processes volume mounts so the anonymous volume "wins" for `/app/node_modules`. The first time the container starts, Docker copies the image's node_modules into the anonymous volume, and from then on it persists there independently.

So the host's node_modules (compiled for macOS) are never used inside the container. The container always uses its own Linux-compiled modules.

### Why thepopebot is installed twice (host and container)

The user runs `npm install` on the host (macOS) to get thepopebot and all dependencies. This is needed because `next build` must resolve all `thepopebot/*` imports to compile the app — without thepopebot in local node_modules, the build fails immediately on unresolved imports. The `.next/` output is just bundled JavaScript — it's platform-independent, so building on macOS and running on Linux is fine. But Next.js still needs `node_modules` at **runtime** for native modules (like `better-sqlite3`) and server-side requires that aren't bundled. Those native modules must be compiled for Linux, which is why the Docker image has its own separate `npm install`. Different purposes, different platforms, both necessary.

### The build must happen before the container starts

Before running `docker-compose up`, the user must run `npm run build` on the host to generate `.next/`. If the container starts without a valid `.next/` build, PM2 will crash-loop with "Could not find a production build" until a build is available. After code changes, `rebuild-event-handler.yml` runs `next build` inside the container via `docker exec` (using the container's node_modules).

## docker-compose.yml Services

| Service | Image | Purpose |
|---------|-------|---------|
| **traefik** | `traefik:v3` | Reverse proxy with automatic HTTPS (Let's Encrypt) |
| **event-handler** | `stephengpope/thepopebot:event-handler-${THEPOPEBOT_VERSION}` | Node.js runtime + PM2, serves the bind-mounted Next.js app on port 80 |
| **runner** | `myoung34/github-runner:latest` | Self-hosted GitHub Actions runner for executing jobs |

The runner registers as a self-hosted GitHub Actions runner, enabling `run-job.yml` to spin up Docker agent containers directly on your server. It also has a read-only volume mount (`.:/project:ro`) so `upgrade-event-handler.yml` can run `docker compose` commands against the project's compose file.

## Deploy to a VPS

Deploy your agent to a cloud VPS with HTTPS.

### 1. Server prerequisites

You need a VPS (any provider — Hetzner, DigitalOcean, AWS, etc.) with:

- Docker + Docker Compose
- Node.js 18+
- Git
- GitHub CLI (`gh`)

Point a domain (e.g., `mybot.example.com`) to your server's IP address with a DNS A record.

### 2. Scaffold and configure

SSH into your server and scaffold the project:

```bash
mkdir my-agent && cd my-agent
npx thepopebot@latest init
npm run setup
```

When the setup wizard asks for `APP_URL`, enter your production URL with `https://` (e.g., `https://mybot.example.com`).

Set the `RUNS_ON` GitHub variable so workflows use your server's self-hosted runner instead of GitHub-hosted runners:

```bash
gh variable set RUNS_ON --body "self-hosted" --repo OWNER/REPO
```

### 3. Enable HTTPS (Let's Encrypt)

The `docker-compose.yml` has Let's Encrypt support built in but commented out. Three edits to enable it:

**a) Add your email to `.env`:**

```
LETSENCRYPT_EMAIL=you@example.com
```

**b) In `docker-compose.yml`, remove the `#` from the TLS lines in the traefik service command:**

```yaml
# Before (commented out):
# - --entrypoints.web.http.redirections.entrypoint.to=websecure
# ...

# After (uncommented):
- --entrypoints.web.http.redirections.entrypoint.to=websecure
- --entrypoints.web.http.redirections.entrypoint.scheme=https
- --certificatesresolvers.letsencrypt.acme.email=${LETSENCRYPT_EMAIL}
- --certificatesresolvers.letsencrypt.acme.storage=/letsencrypt/acme.json
- --certificatesresolvers.letsencrypt.acme.httpchallenge.entrypoint=web
```

**c) In the event-handler labels, switch from HTTP to HTTPS:**

Add a `#` to comment out the HTTP entrypoint, and remove the `#` from the two HTTPS lines:

```yaml
# Before:
- traefik.http.routers.event-handler.entrypoints=web
# - traefik.http.routers.event-handler.entrypoints=websecure
# - traefik.http.routers.event-handler.tls.certresolver=letsencrypt

# After:
# - traefik.http.routers.event-handler.entrypoints=web
- traefik.http.routers.event-handler.entrypoints=websecure
- traefik.http.routers.event-handler.tls.certresolver=letsencrypt
```

### 4. Build and launch

```bash
npm run build
docker compose up -d
```

Ports 80 and 443 must be open on your server. Port 80 is required even with HTTPS — Let's Encrypt uses it for the ACME HTTP challenge to verify domain ownership.
