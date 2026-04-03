"""Agent MAXX — Notifier: Telegram alerts, approval flow, nightly digests."""

import asyncio
import json
import logging
import os
from datetime import datetime, timezone

import httpx
import redis.asyncio as aioredis

logger = logging.getLogger(__name__)

TELEGRAM_BOT_TOKEN = os.getenv("TELEGRAM_BOT_TOKEN", "")
TELEGRAM_CHAT_ID = os.getenv("TELEGRAM_CHAT_ID", "")
REDIS_URL = os.getenv("REDIS_URL", "redis://redis:6379/0")
HERMES_API_URL = os.getenv("HERMES_API_URL", "http://hermes-api:8000")


async def send_telegram(text: str, parse_mode: str = "HTML"):
    if not TELEGRAM_BOT_TOKEN or not TELEGRAM_CHAT_ID: return
    async with httpx.AsyncClient(timeout=15.0) as client:
        try:
            await client.post(f"https://api.telegram.org/bot{TELEGRAM_BOT_TOKEN}/sendMessage",
                              json={"chat_id": TELEGRAM_CHAT_ID, "text": text,
                                    "parse_mode": parse_mode, "disable_web_page_preview": True})
        except Exception as e:
            logger.warning(f"Telegram send failed: {e}")


async def listen_approvals(r):
    pubsub = r.pubsub()
    await pubsub.subscribe("events:approvals")
    async for msg in pubsub.listen():
        if msg["type"] != "message": continue
        try:
            data = json.loads(msg["data"])
            risk = data.get("risk_level", "unknown")
            emoji = {"critical": "🔴", "high": "🟠", "medium": "🟡"}.get(risk, "⚪")
            await send_telegram(
                f"{emoji} <b>Approval Required</b>\nRisk: <code>{risk}</code>\n"
                f"Run: <code>{data.get('run_id', '?')[:8]}</code>\n"
                f"Command: <code>{data.get('proposed_command', '?')[:100]}</code>"
            )
        except Exception as e: logger.warning(f"Approval notify error: {e}")


async def listen_interventions(r):
    pubsub = r.pubsub()
    await pubsub.subscribe("browser:interventions")
    async for msg in pubsub.listen():
        if msg["type"] != "message": continue
        try:
            data = json.loads(msg["data"])
            await send_telegram(
                f"🌐 <b>Browser Intervention</b>\nRun: <code>{data.get('run_id', '?')[:8]}</code>\n"
                f"Challenge: <code>{data.get('challenge_type', '?')}</code>"
            )
        except Exception as e: logger.warning(f"Intervention notify error: {e}")


async def listen_runs(r):
    pubsub = r.pubsub()
    await pubsub.subscribe("events:runs")
    async for msg in pubsub.listen():
        if msg["type"] != "message": continue
        try:
            data = json.loads(msg["data"])
            status = data.get("status", "")
            if status in ("completed", "failed"):
                emoji = "✅" if status == "completed" else "❌"
                await send_telegram(
                    f"{emoji} <b>Run {status}</b>\nID: <code>{data.get('run_id', '?')[:8]}</code>\n"
                    f"Type: {data.get('run_type', '—')}"
                )
        except Exception as e: logger.warning(f"Run notify error: {e}")


async def main():
    logging.basicConfig(level=logging.INFO)
    logger.info("📡 Agent MAXX Notifier starting…")
    r = aioredis.from_url(REDIS_URL)
    asyncio.create_task(listen_approvals(r))
    asyncio.create_task(listen_interventions(r))
    asyncio.create_task(listen_runs(r))
    while True:
        await asyncio.sleep(60)


if __name__ == "__main__":
    asyncio.run(main())
