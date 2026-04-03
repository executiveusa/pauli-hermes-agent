# ============================================================
# Hermes Agent — Production Dockerfile
# ============================================================
# Builds a minimal, production-ready image for running
# Hermes on a VPS (Coolify / Docker / bare Docker).
#
# Exposed port: 8642 (OpenAI-compatible API server)
#
# Build:
#   docker build -t hermes-agent .
#
# Run (with config volume):
#   docker run -d \
#     -p 8642:8642 \
#     -v $HOME/.hermes:/root/.hermes:ro \
#     --env-file .env \
#     hermes-agent
# ============================================================

FROM python:3.11-slim AS base

# ── System deps ────────────────────────────────────────────
RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    curl \
    ripgrep \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# ── Python packaging tools ─────────────────────────────────
RUN pip install --no-cache-dir uv

# ── App working directory ─────────────────────────────────
WORKDIR /app

# ── Copy dependency metadata first (layer cache) ──────────
COPY pyproject.toml requirements.txt ./

# ── Install dependencies (all extras useful for gateway) ──
RUN uv pip install --system --no-cache \
    ".[messaging,mcp,cron,homeassistant,sms,dingtalk,voice]" \
    || pip install --no-cache-dir -e ".[messaging,mcp,cron]"

# ── Copy application source ────────────────────────────────
COPY . .

# ── Install the package itself ─────────────────────────────
RUN pip install --no-cache-dir -e "." --no-deps

# ── Create non-root user (security) ───────────────────────
RUN groupadd -r hermes && useradd -r -g hermes -d /home/hermes -m hermes

# ── Ensure config directory exists ────────────────────────
RUN mkdir -p /home/hermes/.hermes && chown -R hermes:hermes /home/hermes

USER hermes
ENV HOME=/home/hermes
ENV HERMES_HOME=/home/hermes/.hermes

# ── Expose API server port ─────────────────────────────────
EXPOSE 8642

# ── Default: run the gateway (enables API server + messaging) ─
# Override CMD at runtime to switch between modes:
#   gateway mode (default):  python -m gateway.run
#   CLI mode:                python cli.py
#   API server only:         hermes gateway --api-only
CMD ["python", "-m", "gateway.run"]
