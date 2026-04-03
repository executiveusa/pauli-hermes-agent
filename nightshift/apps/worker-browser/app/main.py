"""Agent MAXX — Worker Browser: Playwright automation with state machine."""

import asyncio
import json
import logging
import os
import uuid
from datetime import datetime, timezone
from enum import Enum
from pathlib import Path
from typing import Optional

import httpx
import redis.asyncio as aioredis

logger = logging.getLogger(__name__)

ARTIFACTS_ROOT = Path(os.getenv("ARTIFACTS_ROOT", "/data/artifacts"))
PROVIDER_ROUTER_URL = os.getenv("PROVIDER_ROUTER_URL", "http://provider-router:8080")
REDIS_URL = os.getenv("REDIS_URL", "redis://redis:6379/0")
HERMES_API_URL = os.getenv("HERMES_API_URL", "http://hermes-api:8000")
MAX_CONCURRENT = int(os.getenv("BROWSER_MAX_CONCURRENT", "3"))

CAPTCHA_INDICATORS = ["recaptcha", "hcaptcha", "cf-turnstile", "g-recaptcha", "captcha-container", "arkose"]
MFA_INDICATORS = ["verification code", "two-factor", "2fa", "authenticator", "enter the code", "security code"]


class BrowserState(str, Enum):
    queued = "queued"; starting = "starting"; navigating = "navigating"
    interacting = "interacting"; waiting_intervention = "waiting_intervention"
    paused = "paused"; completed = "completed"; failed = "failed"; cancelled = "cancelled"


async def detect_challenge(page) -> Optional[str]:
    try:
        content = (await page.content()).lower()
        for ind in CAPTCHA_INDICATORS:
            if ind in content: return "captcha"
        for ind in MFA_INDICATORS:
            if ind in content: return "mfa"
    except Exception: pass
    return None


class BrowserSessionRunner:
    def __init__(self, run_id: str, job_data: dict):
        self.run_id = run_id
        self.job = job_data
        self.state = BrowserState.queued
        self.artifacts: list[dict] = []
        self._cancelled = asyncio.Event()
        self._paused = asyncio.Event()
        self._paused.set()

    async def run(self):
        try:
            from playwright.async_api import async_playwright
        except ImportError:
            await self._report("failed", error="playwright not installed")
            return

        self.state = BrowserState.starting
        await self._report("starting")

        try:
            async with async_playwright() as pw:
                browser = await pw.chromium.launch(headless=True, args=["--no-sandbox"])
                context = await browser.new_context(
                    viewport={"width": 1280, "height": 900},
                    record_video_dir=str(ARTIFACTS_ROOT / f"videos/{self.run_id}"),
                )
                page = await context.new_page()
                await context.tracing.start(screenshots=True, snapshots=True)

                try:
                    await self._execute_steps(page)
                finally:
                    trace_path = ARTIFACTS_ROOT / f"traces/{self.run_id}.zip"
                    trace_path.parent.mkdir(parents=True, exist_ok=True)
                    await context.tracing.stop(path=str(trace_path))
                    await context.close()
                    await browser.close()

            if self.state not in (BrowserState.failed, BrowserState.cancelled):
                self.state = BrowserState.completed
                await self._report("completed")
        except Exception as e:
            self.state = BrowserState.failed
            await self._report("failed", error=str(e))
            logger.exception(f"Browser run {self.run_id} failed: {e}")

    async def _execute_steps(self, page):
        steps = self.job.get("steps", [])
        url = self.job.get("start_url", "")

        if url:
            self.state = BrowserState.navigating
            await self._report("navigating")
            await page.goto(url, wait_until="domcontentloaded", timeout=30000)
            await self._screenshot(page, "initial")
            challenge = await detect_challenge(page)
            if challenge:
                await self._handle_challenge(page, challenge)

        self.state = BrowserState.interacting
        await self._report("interacting")

        for i, step in enumerate(steps):
            if self._cancelled.is_set():
                self.state = BrowserState.cancelled
                await self._report("cancelled")
                return
            await self._paused.wait()

            action = step.get("action", "")
            selector = step.get("selector", "")
            value = step.get("value", "")

            try:
                if action == "click": await page.click(selector, timeout=10000)
                elif action == "fill": await page.fill(selector, value, timeout=10000)
                elif action == "type": await page.type(selector, value, delay=50)
                elif action == "select": await page.select_option(selector, value)
                elif action == "wait": await page.wait_for_selector(selector, timeout=int(value or 10000))
                elif action == "screenshot": await self._screenshot(page, f"step_{i}")
                elif action == "navigate": await page.goto(value, wait_until="domcontentloaded", timeout=30000)
                elif action == "extract":
                    text = await page.inner_text(selector) if selector else await page.content()
                    self.artifacts.append({"kind": "extract", "content": text[:5000], "label": f"extract_{i}"})
                elif action == "ai_decide":
                    content = await page.content()
                    decision = await self._ai_decide(content[:4000], value)
                    if decision.get("action"): steps.insert(i + 1, decision)

                await asyncio.sleep(step.get("wait_ms", 500) / 1000.0)
                challenge = await detect_challenge(page)
                if challenge: await self._handle_challenge(page, challenge)
            except Exception as e:
                logger.warning(f"Step {i} ({action}) failed: {e}")
                await self._screenshot(page, f"error_{i}")

        await self._screenshot(page, "final")

    async def _handle_challenge(self, page, challenge_type: str):
        self.state = BrowserState.waiting_intervention
        await self._screenshot(page, f"challenge_{challenge_type}")
        await self._report("waiting_intervention", error=f"{challenge_type} detected")
        r = aioredis.from_url(REDIS_URL)
        await r.publish("browser:interventions", json.dumps({
            "run_id": self.run_id, "challenge_type": challenge_type,
            "timestamp": datetime.now(timezone.utc).isoformat(),
        }))
        await r.close()
        while self.state == BrowserState.waiting_intervention:
            if self._cancelled.is_set(): return
            await asyncio.sleep(5)

    async def _screenshot(self, page, label: str):
        path = ARTIFACTS_ROOT / f"screenshots/{self.run_id}/{label}.png"
        path.parent.mkdir(parents=True, exist_ok=True)
        try: await page.screenshot(path=str(path), full_page=False)
        except Exception: pass

    async def _ai_decide(self, html: str, instruction: str) -> dict:
        try:
            async with httpx.AsyncClient(timeout=60.0) as client:
                resp = await client.post(f"{PROVIDER_ROUTER_URL}/v1/llm/chat", json={
                    "provider_profile_id": "research_browser",
                    "messages": [
                        {"role": "system", "content": 'Output next browser action as JSON: {"action":"...","selector":"...","value":"..."}'},
                        {"role": "user", "content": f"Instruction: {instruction}\n\nHTML:\n{html}"},
                    ], "max_tokens": 512,
                })
            if resp.status_code == 200:
                return json.loads(resp.json().get("response_text", "{}"))
        except Exception: pass
        return {}

    async def _report(self, status: str, error: str = None):
        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                await client.patch(f"{HERMES_API_URL}/api/browser-runs/{self.run_id}",
                                   json={"state": status, "error": error, "artifact_count": len(self.artifacts)})
        except Exception: pass

    def pause(self): self._paused.clear(); self.state = BrowserState.paused
    def resume(self): self._paused.set(); self.state = BrowserState.interacting
    def cancel(self): self._cancelled.set()


_active_runs: dict[str, BrowserSessionRunner] = {}


async def handle_control(redis_client):
    pubsub = redis_client.pubsub()
    await pubsub.subscribe("browser:control")
    async for msg in pubsub.listen():
        if msg["type"] != "message": continue
        try:
            cmd = json.loads(msg["data"])
            runner = _active_runs.get(cmd.get("run_id"))
            if not runner: continue
            action = cmd.get("action")
            if action == "pause": runner.pause()
            elif action == "resume": runner.resume()
            elif action == "cancel": runner.cancel()
        except Exception: pass


async def consume_browser_jobs():
    r = aioredis.from_url(REDIS_URL)
    sem = asyncio.Semaphore(MAX_CONCURRENT)
    logger.info(f"🌐 Worker-Browser listening (max_concurrent={MAX_CONCURRENT})")
    asyncio.create_task(handle_control(r))

    while True:
        try:
            result = await r.brpop("queue:browser_runs", timeout=30)
            if result is None: continue
            _, raw = result
            job = json.loads(raw)
            run_id = job.get("run_id", str(uuid.uuid4()))

            async def run_with_sem():
                async with sem:
                    runner = BrowserSessionRunner(run_id, job)
                    _active_runs[run_id] = runner
                    try: await runner.run()
                    finally: _active_runs.pop(run_id, None)

            asyncio.create_task(run_with_sem())
        except Exception as e:
            logger.exception(f"Browser consumer error: {e}")
            await asyncio.sleep(5)


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    asyncio.run(consume_browser_jobs())
