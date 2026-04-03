"""Agent MAXX — Scheduler: Nightly cron jobs."""

import asyncio
import json
import logging
import os
from datetime import datetime, time, timezone

import httpx
import redis.asyncio as aioredis

logger = logging.getLogger(__name__)

REDIS_URL = os.getenv("REDIS_URL", "redis://redis:6379/0")
HERMES_API_URL = os.getenv("HERMES_API_URL", "http://hermes-api:8000")
SCAN_HOUR = int(os.getenv("SCHEDULER_SCAN_HOUR", "2"))
PRD_HOUR = int(os.getenv("SCHEDULER_PRD_HOUR", "3"))
DIGEST_HOUR = int(os.getenv("SCHEDULER_DIGEST_HOUR", "7"))


async def trigger_repo_scan():
    async with httpx.AsyncClient(timeout=30.0) as client:
        await client.post(f"{HERMES_API_URL}/api/repos/scan-trigger", json={"source": "scheduler"})

async def trigger_prd_batch():
    async with httpx.AsyncClient(timeout=30.0) as client:
        await client.post(f"{HERMES_API_URL}/api/prd-batches/generate", json={"strategy": "nightly_auto"})

async def trigger_digest():
    r = aioredis.from_url(REDIS_URL)
    await r.publish("events:digest", json.dumps({"type": "nightly_digest",
        "timestamp": datetime.now(timezone.utc).isoformat()}))
    await r.close()


JOBS = [
    (SCAN_HOUR, "repo_scan", trigger_repo_scan),
    (PRD_HOUR, "prd_batch", trigger_prd_batch),
    (DIGEST_HOUR, "nightly_digest", trigger_digest),
]


async def run_scheduler():
    logging.basicConfig(level=logging.INFO)
    logger.info(f"🕐 Scheduler: scan={SCAN_HOUR}h, prd={PRD_HOUR}h, digest={DIGEST_HOUR}h UTC")
    last_run: dict[str, str] = {}

    while True:
        now = datetime.now(timezone.utc)
        today = now.strftime("%Y-%m-%d")
        for hour, name, fn in JOBS:
            if now.hour == hour and last_run.get(name) != today:
                logger.info(f"Running: {name}")
                try: await fn()
                except Exception as e: logger.exception(f"Job {name} failed: {e}")
                last_run[name] = today
        await asyncio.sleep(60)


if __name__ == "__main__":
    asyncio.run(run_scheduler())
