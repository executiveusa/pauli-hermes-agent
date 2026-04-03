#!/usr/bin/env python3
"""
Deploy Hermes Agent to Coolify VPS via Coolify API.
Usage: python scripts/deploy_to_coolify.py
"""
import os
import sys
import json
import time
import requests

# ── Load master.env ────────────────────────────────────────────────────────────
MASTER_ENV = r"E:\THE PAULI FILES\master.env"

def load_env(path):
    env = {}
    try:
        with open(path) as f:
            for line in f:
                s = line.strip()
                if "=" in s and not s.startswith("#"):
                    k, v = s.split("=", 1)
                    env[k.strip()] = v.strip()
    except FileNotFoundError:
        print(f"[ERROR] master.env not found at {path}")
        sys.exit(1)
    return env

env = load_env(MASTER_ENV)

# ── Config ─────────────────────────────────────────────────────────────────────
COOLIFY_URL   = env.get("COOLIFY_URL", "http://31.220.58.212:8000").rstrip("/")
COOLIFY_TOKEN = env.get("COOLIFY_API_TOKEN", "")
PROJECT_ID    = env.get("COOLIFY_PROJECT_ID", "")

HERMES_REPO   = "https://github.com/executiveusa/pauli-hermes-agent.git"
HERMES_BRANCH = "main"

# Env vars to inject into the Hermes deployment
HERMES_ENV_KEYS = [
    "ANTHROPIC_API_KEY",
    "OPENAI_API_KEY",
    "GEMINI_API_KEY",
    "GROQ_API_KEY",
    "DEEPSEEK_API_KEY",
    "TAVILY_API_KEY",
    "FIRECRAWL_API_KEY",
    "BRAVE_API_KEY",
    "TELEGRAM_BOT_TOKEN",
    "DISCORD_BOT_TOKEN",
    "SUPABASE_URL",
    "SUPABASE_KEY",
    "SUPABASE_SERVICE_KEY",
    "DATABASE_URL",
]

if not COOLIFY_TOKEN:
    print("[ERROR] COOLIFY_API_TOKEN not found in master.env")
    sys.exit(1)

HEADERS = {
    "Authorization": f"Bearer {COOLIFY_TOKEN}",
    "Content-Type": "application/json",
    "Accept": "application/json",
}

# ── Supabase defaults (from connection doc) ────────────────────────────────────
SUPABASE_DEFAULTS = {
    "SUPABASE_URL": "http://31.220.58.212:8001",
    "DATABASE_URL": "postgresql://postgres:072090156d28a9df6502d94083e47990@31.220.58.212:5434/second_brain",
}

def get(path):
    r = requests.get(f"{COOLIFY_URL}/api/v1{path}", headers=HEADERS, timeout=15)
    return r

def post(path, data):
    r = requests.post(f"{COOLIFY_URL}/api/v1{path}", headers=HEADERS, json=data, timeout=30)
    return r

# ── Step 1: Test connection ────────────────────────────────────────────────────
print(f"\n[1] Testing Coolify connection at {COOLIFY_URL} ...")
r = get("/version")
if r.status_code != 200:
    print(f"[WARN] /version returned {r.status_code}. Trying /healthcheck ...")
    r = get("/healthcheck")
    if r.status_code not in (200, 201):
        print(f"[ERROR] Cannot reach Coolify API: {r.status_code} {r.text[:200]}")
        sys.exit(1)
print(f"[OK] Coolify reachable. Response: {r.text[:120]}")

# ── Step 2: List servers ────────────────────────────────────────────────────────
print("\n[2] Listing servers ...")
r = get("/servers")
if r.status_code != 200:
    print(f"[ERROR] Cannot list servers: {r.status_code} - {r.text[:200]}")
    sys.exit(1)
servers = r.json()
if not servers:
    print("[ERROR] No servers found in Coolify. Is the VPS connected?")
    sys.exit(1)

server = servers[0]
server_uuid = server.get("uuid") or server.get("id")
print(f"[OK] Using server: {server.get('name', server_uuid)} (uuid={server_uuid})")

# ── Step 3: Get or create project ──────────────────────────────────────────────
print("\n[3] Resolving project ...")
r = get("/projects")
projects = r.json() if r.status_code == 200 else []
hermes_project = None
for p in projects:
    if "hermes" in p.get("name", "").lower():
        hermes_project = p
        break

if not hermes_project:
    print("   Creating 'Hermes Agent' project ...")
    r = post("/projects", {"name": "Hermes Agent", "description": "Hermes AI Agent stack"})
    if r.status_code not in (200, 201):
        print(f"[ERROR] Failed to create project: {r.status_code} {r.text[:200]}")
        sys.exit(1)
    hermes_project = r.json()

project_uuid = hermes_project.get("uuid") or hermes_project.get("id")
print(f"[OK] Project: {hermes_project.get('name')} (uuid={project_uuid})")

# ── Step 4: Get environment (default) ──────────────────────────────────────────
print("\n[4] Getting project environments ...")
r = get(f"/projects/{project_uuid}/environments")
if r.status_code != 200:
    print(f"[WARN] Could not fetch environments: {r.status_code}")
    env_name = "production"
else:
    envs = r.json()
    env_name = envs[0].get("name", "production") if envs else "production"
print(f"[OK] Environment: {env_name}")

# ── Step 5: Build env dict for Hermes ──────────────────────────────────────────
hermes_env_vars = {}
for key in HERMES_ENV_KEYS:
    val = env.get(key) or SUPABASE_DEFAULTS.get(key, "")
    if val:
        hermes_env_vars[key] = val

# Mandatory Hermes settings
hermes_env_vars.update({
    "API_SERVER_ENABLED": "1",
    "API_SERVER_HOST": "0.0.0.0",
    "API_SERVER_PORT": "8642",
    "API_SERVER_KEY": env.get("API_SERVER_KEY", "hermes-change-me-now"),
    "HERMES_PLATFORM": "api",
    "HERMES_DEFAULT_MODEL": "anthropic/claude-sonnet-4-5",
    "PYTHONUNBUFFERED": "1",
})

# Format as Coolify env_vars array
env_vars_list = "\n".join([f"{k}={v}" for k, v in hermes_env_vars.items()])
print(f"\n[5] Prepared {len(hermes_env_vars)} env vars for Hermes")

# ── Step 6: Check if Hermes app already exists ─────────────────────────────────
print("\n[6] Checking for existing Hermes application ...")
r = get("/applications")
apps = r.json() if r.status_code == 200 else []
existing_app = None
for app in (apps if isinstance(apps, list) else []):
    name = app.get("name", "")
    if "hermes" in name.lower():
        existing_app = app
        break

# ── Step 7: Create or redeploy ─────────────────────────────────────────────────
if existing_app:
    app_uuid = existing_app.get("uuid") or existing_app.get("id")
    print(f"[OK] Found existing app: {existing_app.get('name')} (uuid={app_uuid})")
    print("\n[7] Triggering redeploy ...")
    r = post(f"/applications/{app_uuid}/restart", {})
    if r.status_code in (200, 201):
        print("[OK] Redeploy triggered!")
    else:
        print(f"[WARN] Restart returned {r.status_code}: {r.text[:200]}")
        print("   Trying deploy endpoint ...")
        r = post(f"/applications/{app_uuid}/deploy", {"force_rebuild": True})
        print(f"   Deploy response: {r.status_code} {r.text[:120]}")
else:
    print("[7] Creating new Hermes application from GitHub ...")
    payload = {
        "project_uuid": project_uuid,
        "server_uuid": server_uuid,
        "environment_name": env_name,
        "git_repository": HERMES_REPO,
        "git_branch": HERMES_BRANCH,
        "build_pack": "dockerfile",
        "dockerfile_location": "/Dockerfile",
        "name": "hermes-agent",
        "description": "Hermes AI Agent — API + Gateway",
        "ports_exposes": "8642",
        "ports_mappings": "8642:8642",
        "instant_deploy": True,
        "environment_variables": env_vars_list,
    }
    r = post("/applications", payload)
    if r.status_code in (200, 201):
        result = r.json()
        app_uuid = result.get("uuid") or result.get("id", "unknown")
        print(f"[OK] Application created! uuid={app_uuid}")
        print(f"     Deploy triggered automatically (instant_deploy=True)")
    else:
        print(f"[ERROR] Failed to create application: {r.status_code}")
        print(r.text[:500])
        sys.exit(1)

# ── Step 8: Also deploy dashboard (web-ui) ─────────────────────────────────────
print("\n[8] Checking for dashboard application ...")
dashboard_exists = any(
    "dashboard" in (a.get("name","")).lower() or "web-ui" in (a.get("name","")).lower()
    for a in (apps if isinstance(apps, list) else [])
)

if not dashboard_exists:
    print("   Creating dashboard (voice cockpit) app ...")
    dash_payload = {
        "project_uuid": project_uuid,
        "server_uuid": server_uuid,
        "environment_name": env_name,
        "git_repository": HERMES_REPO,
        "git_branch": HERMES_BRANCH,
        "build_pack": "static",
        "static_image": "nginx:alpine",
        "publish_directory": "/web-ui",
        "name": "hermes-dashboard",
        "description": "Hermes Voice Control Cockpit",
        "ports_exposes": "80",
        "ports_mappings": "9000:80",
        "instant_deploy": True,
        "environment_variables": f"HERMES_API_URL=http://31.220.58.212:8642\nHERMES_API_KEY={hermes_env_vars.get('API_SERVER_KEY','')}"
    }
    r = post("/applications", dash_payload)
    if r.status_code in (200, 201):
        print(f"[OK] Dashboard app created!")
    else:
        print(f"[WARN] Dashboard creation: {r.status_code} {r.text[:200]}")
else:
    print("[OK] Dashboard already exists, skipping.")

# ── Done ────────────────────────────────────────────────────────────────────────
print("\n" + "="*60)
print("DEPLOYMENT SUMMARY")
print("="*60)
print(f"  VPS IP:          31.220.58.212")
print(f"  Coolify Panel:   http://31.220.58.212:8000")
print(f"  Hermes API:      http://31.220.58.212:8642")
print(f"  Dashboard:       http://31.220.58.212:9000")
print(f"  Supabase Studio: http://31.220.58.212:3001")
print("="*60)
print("\nMonitor build progress in Coolify panel: http://31.220.58.212:8000")
