#!/usr/bin/env python3
"""
Strix Batch Security Testing - Parallel agent orchestration for multi-repo testing.
Spawns multiple Strix agents to test GitHub repos for vulnerabilities simultaneously.
"""

import json
import os
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass, asdict
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Optional
import time


@dataclass
class Target:
    """Represents a testing target (repo, URL, or API spec)."""
    url: str
    name: Optional[str] = None
    type: str = "github"  # github, url, openapi, postman
    auth_credentials: Optional[Dict] = None
    custom_instructions: Optional[str] = None

    def __post_init__(self):
        if not self.name:
            self.name = self.url.split("/")[-1].replace(".git", "")


@dataclass
class ScanResult:
    """Results from a single security scan."""
    target: str
    status: str  # success, error, timeout
    start_time: str
    end_time: str
    duration_seconds: float
    vulnerabilities_found: int
    critical_count: int
    high_count: int
    medium_count: int
    low_count: int
    info_count: int
    error_message: Optional[str] = None
    report_path: Optional[str] = None
    findings_file: Optional[str] = None


class StrixBatchTester:
    """Orchestrates parallel security testing across multiple targets."""

    def __init__(
        self,
        max_workers: int = 3,
        output_dir: str = "./security-scans",
        llm_provider: Optional[str] = None,
        llm_api_key: Optional[str] = None,
        reasoning_effort: str = "high",
        scan_mode: str = "standard",  # quick, standard, full
    ):
        self.max_workers = max_workers
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)

        self.llm_provider = llm_provider or os.getenv("STRIX_LLM", "openai/gpt-5.4")
        self.llm_api_key = llm_api_key or os.getenv("LLM_API_KEY")
        self.reasoning_effort = reasoning_effort
        self.scan_mode = scan_mode

        self.results: List[ScanResult] = []
        self.timestamp = datetime.now().isoformat()

    def _check_strix_installed(self) -> bool:
        """Verify Strix is installed and accessible."""
        try:
            result = subprocess.run(
                ["strix", "--version"],
                capture_output=True,
                text=True,
                timeout=5,
            )
            return result.returncode == 0
        except (FileNotFoundError, subprocess.TimeoutExpired):
            return False

    def _build_strix_command(self, target: Target) -> List[str]:
        """Build the strix CLI command for a target."""
        cmd = [
            "strix",
            "-n",  # non-interactive
            "--target", target.url,
        ]

        if self.scan_mode:
            cmd.extend(["--scan-mode", self.scan_mode])

        if target.custom_instructions:
            cmd.extend(["--instruction", target.custom_instructions])

        if target.auth_credentials:
            auth_str = f"username:{target.auth_credentials.get('username')}/password:{target.auth_credentials.get('password')}"
            cmd.extend(["--instruction", f"Use credentials: {auth_str}"])

        return cmd

    def _parse_strix_output(self, stdout: str, stderr: str) -> Dict:
        """Parse Strix output to extract findings."""
        findings = {
            "critical": 0,
            "high": 0,
            "medium": 0,
            "low": 0,
            "info": 0,
            "total": 0,
        }

        # Parse severity counts from output
        for line in (stdout + stderr).split("\n"):
            if "CRITICAL" in line:
                findings["critical"] += 1
            elif "HIGH" in line:
                findings["high"] += 1
            elif "MEDIUM" in line:
                findings["medium"] += 1
            elif "LOW" in line:
                findings["low"] += 1
            elif "INFO" in line:
                findings["info"] += 1

        findings["total"] = sum(v for k, v in findings.items() if k != "total")
        return findings

    def _test_single_target(self, target: Target) -> ScanResult:
        """Run security test on a single target."""
        start_time = datetime.now()
        start_iso = start_time.isoformat()

        target_output_dir = self.output_dir / target.name
        target_output_dir.mkdir(parents=True, exist_ok=True)

        try:
            print(f"[{start_time.strftime('%H:%M:%S')}] Testing {target.url}...")

            cmd = self._build_strix_command(target)

            # Set environment variables
            env = os.environ.copy()
            env["STRIX_LLM"] = self.llm_provider
            if self.llm_api_key:
                env["LLM_API_KEY"] = self.llm_api_key
            env["STRIX_REASONING_EFFORT"] = self.reasoning_effort

            # Run Strix
            process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                env=env,
                cwd=str(target_output_dir),
            )

            try:
                stdout, stderr = process.communicate(timeout=3600)  # 1 hour timeout
            except subprocess.TimeoutExpired:
                process.kill()
                raise TimeoutError(f"Strix timeout after 3600 seconds for {target.url}")

            # Parse results
            findings = self._parse_strix_output(stdout, stderr)

            # Find findings.json if it exists
            findings_file = None
            for run_dir in target_output_dir.glob("strix_runs/*/findings.json"):
                findings_file = str(run_dir)
                break

            end_time = datetime.now()
            duration = (end_time - start_time).total_seconds()

            status = "success" if process.returncode in (0, 1) else "error"

            result = ScanResult(
                target=target.url,
                status=status,
                start_time=start_iso,
                end_time=end_time.isoformat(),
                duration_seconds=duration,
                vulnerabilities_found=findings["total"],
                critical_count=findings["critical"],
                high_count=findings["high"],
                medium_count=findings["medium"],
                low_count=findings["low"],
                info_count=findings["info"],
                report_path=str(target_output_dir),
                findings_file=findings_file,
            )

            print(f"✓ Completed {target.url} - Found {findings['total']} vulnerabilities")
            return result

        except Exception as e:
            end_time = datetime.now()
            duration = (end_time - start_time).total_seconds()

            print(f"✗ Failed testing {target.url}: {str(e)}")

            return ScanResult(
                target=target.url,
                status="error",
                start_time=start_iso,
                end_time=end_time.isoformat(),
                duration_seconds=duration,
                vulnerabilities_found=0,
                critical_count=0,
                high_count=0,
                medium_count=0,
                low_count=0,
                info_count=0,
                error_message=str(e),
                report_path=str(target_output_dir),
            )

    def test_targets(self, targets: List[Target]) -> List[ScanResult]:
        """Test multiple targets in parallel."""
        if not self._check_strix_installed():
            print("✗ Strix is not installed. Install with: curl -sSL https://strix.ai/install | bash")
            sys.exit(1)

        print(f"\n🔐 Starting batch security testing ({len(targets)} targets)")
        print(f"   Max parallel workers: {self.max_workers}")
        print(f"   Scan mode: {self.scan_mode}")
        print(f"   Reasoning effort: {self.reasoning_effort}\n")

        self.results = []

        with ThreadPoolExecutor(max_workers=self.max_workers) as executor:
            futures = {
                executor.submit(self._test_single_target, target): target
                for target in targets
            }

            for future in as_completed(futures):
                result = future.result()
                self.results.append(result)

        return self.results

    def generate_report(self) -> Dict:
        """Generate a comprehensive batch testing report."""
        if not self.results:
            return {}

        total_vulnerabilities = sum(r.vulnerabilities_found for r in self.results)
        total_critical = sum(r.critical_count for r in self.results)
        total_high = sum(r.high_count for r in self.results)
        total_duration = sum(r.duration_seconds for r in self.results)
        success_count = sum(1 for r in self.results if r.status == "success")

        report = {
            "timestamp": self.timestamp,
            "batch_summary": {
                "total_targets": len(self.results),
                "successful_scans": success_count,
                "failed_scans": len(self.results) - success_count,
                "total_vulnerabilities": total_vulnerabilities,
                "severity_breakdown": {
                    "critical": total_critical,
                    "high": total_high,
                    "medium": sum(r.medium_count for r in self.results),
                    "low": sum(r.low_count for r in self.results),
                    "info": sum(r.info_count for r in self.results),
                },
                "total_duration_seconds": total_duration,
                "average_duration_per_target": total_duration / len(self.results) if self.results else 0,
            },
            "scan_configuration": {
                "llm_provider": self.llm_provider,
                "reasoning_effort": self.reasoning_effort,
                "scan_mode": self.scan_mode,
                "max_parallel_workers": self.max_workers,
            },
            "results_by_target": [asdict(r) for r in self.results],
        }

        return report

    def save_report(self, report: Dict, filename: str = "batch-report.json") -> Path:
        """Save report to JSON file."""
        report_path = self.output_dir / filename
        with open(report_path, "w") as f:
            json.dump(report, f, indent=2)
        return report_path

    def print_summary(self):
        """Print a human-readable summary of results."""
        if not self.results:
            print("No results to display.")
            return

        report = self.generate_report()
        summary = report["batch_summary"]

        print("\n" + "=" * 70)
        print("BATCH SECURITY TESTING SUMMARY")
        print("=" * 70)
        print(f"Scan Date: {self.timestamp}")
        print(f"\n📊 Overall Results:")
        print(f"   Total Targets: {summary['total_targets']}")
        print(f"   Successful Scans: {summary['successful_scans']}")
        print(f"   Failed Scans: {summary['failed_scans']}")
        print(f"   Total Vulnerabilities: {summary['total_vulnerabilities']}")
        print(f"\n🔴 Severity Breakdown:")
        print(f"   🚨 Critical: {summary['severity_breakdown']['critical']}")
        print(f"   ⚠️  High: {summary['severity_breakdown']['high']}")
        print(f"   ⚡ Medium: {summary['severity_breakdown']['medium']}")
        print(f"   ℹ️  Low: {summary['severity_breakdown']['low']}")
        print(f"   💡 Info: {summary['severity_breakdown']['info']}")
        print(f"\n⏱️  Performance:")
        print(f"   Total Time: {summary['total_duration_seconds']:.1f}s")
        print(f"   Avg per Target: {summary['average_duration_per_target']:.1f}s")

        print(f"\n📋 Per-Target Results:")
        print("-" * 70)

        for result in self.results:
            status_icon = "✓" if result.status == "success" else "✗"
            print(f"{status_icon} {result.target}")
            if result.status == "success":
                print(f"   Vulnerabilities: {result.vulnerabilities_found} "
                      f"(🚨{result.critical_count} ⚠️{result.high_count} ⚡{result.medium_count})")
            else:
                print(f"   Error: {result.error_message}")
            print(f"   Duration: {result.duration_seconds:.1f}s")

        print("\n" + "=" * 70)


def main():
    """CLI entry point for batch security testing."""
    import argparse

    parser = argparse.ArgumentParser(
        description="Batch security testing using Strix AI agents"
    )
    parser.add_argument(
        "--targets",
        nargs="+",
        required=True,
        help="Target URLs or GitHub repos to test",
    )
    parser.add_argument(
        "--target-file",
        type=str,
        help="File with targets (one per line)",
    )
    parser.add_argument(
        "--output-dir",
        default="./security-scans",
        help="Output directory for scan results",
    )
    parser.add_argument(
        "--workers",
        type=int,
        default=3,
        help="Number of parallel test agents",
    )
    parser.add_argument(
        "--scan-mode",
        choices=["quick", "standard", "full"],
        default="standard",
        help="Scan scope and depth",
    )
    parser.add_argument(
        "--reasoning-effort",
        choices=["quick", "medium", "high"],
        default="high",
        help="LLM reasoning effort level",
    )
    parser.add_argument(
        "--llm-provider",
        help="LLM provider (e.g., openai/gpt-5.4, anthropic/claude-sonnet-4-6)",
    )
    parser.add_argument(
        "--llm-api-key",
        help="LLM API key (or set LLM_API_KEY env var)",
    )
    parser.add_argument(
        "--save-report",
        action="store_true",
        help="Save JSON report to file",
    )

    args = parser.parse_args()

    # Load targets
    targets = []

    if args.targets:
        for url in args.targets:
            targets.append(Target(url=url))

    if args.target_file:
        with open(args.target_file) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#"):
                    targets.append(Target(url=line))

    if not targets:
        parser.error("No targets provided")

    # Run batch testing
    tester = StrixBatchTester(
        max_workers=args.workers,
        output_dir=args.output_dir,
        llm_provider=args.llm_provider,
        llm_api_key=args.llm_api_key,
        reasoning_effort=args.reasoning_effort,
        scan_mode=args.scan_mode,
    )

    tester.test_targets(targets)
    tester.print_summary()

    if args.save_report:
        report = tester.generate_report()
        report_path = tester.save_report(report)
        print(f"\n📄 Report saved to: {report_path}")

    sys.exit(0)


if __name__ == "__main__":
    main()
