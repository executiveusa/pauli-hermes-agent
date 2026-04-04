# /api — External API Routes

This directory contains the route handlers for all `/api/*` endpoints. These routes are for **external callers only** — GitHub Actions, Telegram, cURL, third-party webhooks.

## Auth

All routes (except `/telegram/webhook` and `/github/webhook`, which use their own webhook secrets) require a valid API key passed via the `x-api-key` header. API keys are stored in the SQLite database and managed through the admin UI — they are NOT environment variables.

Auth flow: `x-api-key` header -> `verifyApiKey()` -> database lookup (hashed, timing-safe comparison).

## Do NOT use these routes for browser UI

Browser-facing features must use **Server Actions** (`'use server'` functions) with `requireAuth()` session checks — never `/api` fetch calls. The only exception is chat streaming, which has its own dedicated route at `/stream/chat` with session auth.

| Caller | Mechanism | Auth |
|--------|-----------|------|
| External (cURL, GitHub Actions, Telegram) | `/api` route | `x-api-key` header |
| Browser UI (data/mutations) | Server Action | `requireAuth()` session |
| Browser UI (chat streaming) | `/stream/chat` | `auth()` session |
