"""Cost tracking and usage monitoring for FREE MODE.

Tracks:
- Requests per provider
- Token usage (input/output)
- Estimated costs
- Daily/monthly totals
"""

import json
import logging
from dataclasses import dataclass, asdict
from datetime import datetime, timedelta
from pathlib import Path
from typing import Optional

logger = logging.getLogger(__name__)


@dataclass
class ProviderCost:
    """Cost per provider."""

    provider: str
    requests: int
    input_tokens: int
    output_tokens: int
    estimated_cost: float


@dataclass
class DailyCost:
    """Daily cost aggregate."""

    date: str  # YYYY-MM-DD
    providers: list[ProviderCost]
    total_cost: float
    total_requests: int


# Pricing data (estimate)
PROVIDER_PRICING = {
    "groq": {"input": 0.00005, "output": 0.00015},  # Groq pricing per 1k tokens
    "gemini": {"input": 0.0, "output": 0.0},  # Free tier
    "openrouter": {"input": 0.00005, "output": 0.00015},  # Varies
    "nvidia_nim": {"input": 0.0, "output": 0.0},  # Free tier
    "openai": {"input": 0.0005, "output": 0.0015},  # GPT-4 pricing
    "anthropic": {"input": 0.0003, "output": 0.001},  # Claude pricing
    "mistral": {"input": 0.0001, "output": 0.0003},
    "ollama": {"input": 0.0, "output": 0.0},  # Local
    "lmstudio": {"input": 0.0, "output": 0.0},  # Local
}


class CostTracker:
    """Track costs and usage across providers."""

    def __init__(self, data_dir: Optional[Path] = None):
        self.data_dir = data_dir or Path.home() / ".hermes" / "cost_tracking"
        self.data_dir.mkdir(parents=True, exist_ok=True)
        self.cost_file = self.data_dir / "costs.json"
        self.history_file = self.data_dir / "history.json"
        self._ensure_files()

    def _ensure_files(self):
        """Create cost tracking files if they don't exist."""
        if not self.cost_file.exists():
            self.cost_file.write_text(
                json.dumps(
                    {
                        "daily": {},
                        "monthly": {},
                        "providers": {},
                        "total_requests": 0,
                        "last_updated": datetime.now().isoformat(),
                    }
                )
            )
        if not self.history_file.exists():
            self.history_file.write_text(json.dumps({"entries": []}))

    def track_request(
        self,
        provider: str,
        input_tokens: int = 0,
        output_tokens: int = 0,
    ) -> None:
        """Track a request to a provider.

        Args:
            provider: Provider ID (groq, gemini, openai, etc.)
            input_tokens: Number of input tokens
            output_tokens: Number of output tokens
        """
        try:
            data = json.loads(self.cost_file.read_text())
            pricing = PROVIDER_PRICING.get(provider, {"input": 0.0, "output": 0.0})

            # Calculate cost
            cost = (input_tokens * pricing["input"] + output_tokens * pricing["output"]) / 1000

            # Get date keys
            today = datetime.now().isoformat()[:10]
            month = datetime.now().isoformat()[:7]

            # Update daily cost
            if today not in data["daily"]:
                data["daily"][today] = 0.0
            data["daily"][today] += cost

            # Update monthly cost
            if month not in data["monthly"]:
                data["monthly"][month] = 0.0
            data["monthly"][month] += cost

            # Update provider stats
            if provider not in data["providers"]:
                data["providers"][provider] = {
                    "requests": 0,
                    "input_tokens": 0,
                    "output_tokens": 0,
                    "cost": 0.0,
                }
            data["providers"][provider]["requests"] += 1
            data["providers"][provider]["input_tokens"] += input_tokens
            data["providers"][provider]["output_tokens"] += output_tokens
            data["providers"][provider]["cost"] += cost

            # Update totals
            data["total_requests"] += 1
            data["last_updated"] = datetime.now().isoformat()

            self.cost_file.write_text(json.dumps(data, indent=2))

            # Add to history
            self._add_history_entry(
                provider=provider,
                requests=1,
                cost=cost,
                input_tokens=input_tokens,
                output_tokens=output_tokens,
            )
        except Exception as err:
            logger.error(f"Failed to track request: {err}")

    def _add_history_entry(
        self,
        provider: str,
        requests: int,
        cost: float,
        input_tokens: int = 0,
        output_tokens: int = 0,
    ) -> None:
        """Add entry to cost history."""
        try:
            data = json.loads(self.history_file.read_text())
            entry = {
                "timestamp": datetime.now().isoformat(),
                "provider": provider,
                "requests": requests,
                "cost": cost,
                "input_tokens": input_tokens,
                "output_tokens": output_tokens,
            }
            data["entries"].append(entry)
            # Keep last 1000 entries
            if len(data["entries"]) > 1000:
                data["entries"] = data["entries"][-1000:]
            self.history_file.write_text(json.dumps(data, indent=2))
        except Exception as err:
            logger.error(f"Failed to add history entry: {err}")

    def get_costs_today(self) -> float:
        """Get total cost today."""
        try:
            data = json.loads(self.cost_file.read_text())
            today = datetime.now().isoformat()[:10]
            return data["daily"].get(today, 0.0)
        except Exception:
            return 0.0

    def get_costs_month(self) -> float:
        """Get total cost this month."""
        try:
            data = json.loads(self.cost_file.read_text())
            month = datetime.now().isoformat()[:7]
            return data["monthly"].get(month, 0.0)
        except Exception:
            return 0.0

    def get_provider_stats(self, provider: str) -> Optional[dict]:
        """Get stats for a specific provider."""
        try:
            data = json.loads(self.cost_file.read_text())
            return data["providers"].get(provider)
        except Exception:
            return None

    def get_all_stats(self) -> dict:
        """Get all cost tracking data."""
        try:
            return json.loads(self.cost_file.read_text())
        except Exception:
            return {}

    def get_history(self, days: int = 1) -> list[dict]:
        """Get cost history for past N days."""
        try:
            data = json.loads(self.history_file.read_text())
            cutoff = datetime.now() - timedelta(days=days)
            return [
                e
                for e in data["entries"]
                if datetime.fromisoformat(e["timestamp"]) >= cutoff
            ]
        except Exception:
            return []

    def reset_daily(self) -> None:
        """Reset daily counters (for testing)."""
        try:
            data = json.loads(self.cost_file.read_text())
            data["daily"] = {}
            data["last_updated"] = datetime.now().isoformat()
            self.cost_file.write_text(json.dumps(data, indent=2))
        except Exception as err:
            logger.error(f"Failed to reset daily: {err}")


# Global cost tracker instance
_tracker: Optional[CostTracker] = None


def get_cost_tracker() -> CostTracker:
    """Get or create global cost tracker."""
    global _tracker
    if _tracker is None:
        _tracker = CostTracker()
    return _tracker


def track_request(
    provider: str,
    input_tokens: int = 0,
    output_tokens: int = 0,
) -> None:
    """Track a request globally."""
    get_cost_tracker().track_request(provider, input_tokens, output_tokens)
