#!/usr/bin/env python3
"""
GitHub Multi-Agent Security Testing - Spawn parallel Strix agents to test multiple GitHub repos.
Automatically discovers repos in an organization and tests them for vulnerabilities.
"""

import json
import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import List, Optional
from concurrent.futures import ThreadPoolExecutor, as_completed


@dataclass
class GithubRepo:
    """GitHub repository metadata."""
    owner: str
    repo: str
    url: str
    language: Optional[str] = None
    is_private: bool = False

    @property
    def full_name(self) -> str:
        return f"{self.owner}/{self.repo}"


class GitHubMultiAgentTester:
    """Discovers and tests GitHub repos using parallel Strix agents."""

    def __init__(self, github_token: Optional[str] = None, max_workers: int = 3):
        self.github_token = github_token or os.getenv("GITHUB_TOKEN")
        self.max_workers = max_workers
        self.repos: List[GithubRepo] = []

    def discover_org_repos(self, org: str, language: Optional[str] = None) -> List[GithubRepo]:
        """Discover all repos in a GitHub organization."""
        print(f"🔍 Discovering repos in organization: {org}")

        repos = []

        try:
            import requests
        except ImportError:
            print("⚠️  requests library required. Install with: pip install requests")
            return repos

        headers = {}
        if self.github_token:
            headers["Authorization"] = f"Bearer {self.github_token}"

        page = 1
        while True:
            url = f"https://api.github.com/orgs/{org}/repos"
            params = {
                "page": page,
                "per_page": 100,
                "sort": "updated",
            }

            try:
                resp = requests.get(url, headers=headers, params=params, timeout=10)
                resp.raise_for_status()

                data = resp.json()
                if not data:
                    break

                for repo_data in data:
                    if language and repo_data.get("language") != language:
                        continue

                    repo = GithubRepo(
                        owner=org,
                        repo=repo_data["name"],
                        url=repo_data["html_url"],
                        language=repo_data.get("language"),
                        is_private=repo_data.get("private", False),
                    )
                    repos.append(repo)
                    print(f"   Found: {repo.full_name} ({repo.language})")

                page += 1

            except Exception as e:
                print(f"   Error fetching repos: {e}")
                break

        self.repos = repos
        print(f"✓ Discovered {len(repos)} repositories")
        return repos

    def discover_user_repos(self, username: str) -> List[GithubRepo]:
        """Discover all repos for a GitHub user."""
        print(f"🔍 Discovering repos for user: {username}")

        repos = []

        try:
            import requests
        except ImportError:
            print("⚠️  requests library required. Install with: pip install requests")
            return repos

        headers = {}
        if self.github_token:
            headers["Authorization"] = f"Bearer {self.github_token}"

        page = 1
        while True:
            url = f"https://api.github.com/users/{username}/repos"
            params = {
                "page": page,
                "per_page": 100,
                "sort": "updated",
            }

            try:
                resp = requests.get(url, headers=headers, params=params, timeout=10)
                resp.raise_for_status()

                data = resp.json()
                if not data:
                    break

                for repo_data in data:
                    repo = GithubRepo(
                        owner=username,
                        repo=repo_data["name"],
                        url=repo_data["html_url"],
                        language=repo_data.get("language"),
                        is_private=repo_data.get("private", False),
                    )
                    repos.append(repo)
                    print(f"   Found: {repo.full_name}")

                page += 1

            except Exception as e:
                print(f"   Error fetching repos: {e}")
                break

        self.repos = repos
        print(f"✓ Discovered {len(repos)} repositories")
        return repos

    def test_repo(self, repo: GithubRepo, output_dir: Path) -> dict:
        """Test a single repository with Strix."""
        print(f"🔐 Testing: {repo.full_name}")

        repo_dir = output_dir / repo.repo
        repo_dir.mkdir(parents=True, exist_ok=True)

        try:
            # Clone or update repo
            repo_path = repo_dir / "source"

            if not repo_path.exists():
                print(f"   📥 Cloning {repo.url}...")
                result = subprocess.run(
                    ["git", "clone", "--depth", "1", repo.url, str(repo_path)],
                    capture_output=True,
                    text=True,
                    timeout=300,
                )
                if result.returncode != 0:
                    return {
                        "repo": repo.full_name,
                        "status": "error",
                        "error": f"Failed to clone: {result.stderr}",
                    }

            # Run Strix security test
            print(f"   🔍 Scanning {repo.full_name}...")

            cmd = [
                "strix",
                "-n",
                "--target", str(repo_path),
                "--scan-mode", "quick",  # Use quick mode for batch testing
            ]

            env = os.environ.copy()
            env["STRIX_REASONING_EFFORT"] = "medium"

            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=1800,  # 30 min per repo
                env=env,
                cwd=str(repo_dir),
            )

            # Parse findings
            findings_count = result.stdout.count("CRITICAL") + result.stdout.count("HIGH")

            return {
                "repo": repo.full_name,
                "status": "success",
                "language": repo.language,
                "findings_count": findings_count,
                "output_dir": str(repo_dir),
                "exit_code": result.returncode,
            }

        except subprocess.TimeoutExpired:
            return {
                "repo": repo.full_name,
                "status": "timeout",
                "error": "Test exceeded 30 minute timeout",
            }
        except Exception as e:
            return {
                "repo": repo.full_name,
                "status": "error",
                "error": str(e),
            }

    def batch_test_repos(
        self,
        repos: Optional[List[GithubRepo]] = None,
        output_dir: str = "./github-security-scans",
    ) -> List[dict]:
        """Test multiple repos in parallel."""
        if repos is None:
            repos = self.repos

        if not repos:
            print("No repositories to test")
            return []

        output_path = Path(output_dir)
        output_path.mkdir(parents=True, exist_ok=True)

        print(f"\n🚀 Starting parallel testing ({len(repos)} repos, {self.max_workers} workers)")
        print("=" * 70)

        results = []

        with ThreadPoolExecutor(max_workers=self.max_workers) as executor:
            futures = {
                executor.submit(self.test_repo, repo, output_path): repo
                for repo in repos
            }

            completed = 0
            for future in as_completed(futures):
                result = future.result()
                results.append(result)
                completed += 1

                status_icon = "✓" if result["status"] == "success" else "✗"
                print(f"{status_icon} [{completed}/{len(repos)}] {result['repo']}")

        return results

    def create_github_issues(self, results: List[dict]):
        """Create GitHub issues for critical findings."""
        print("\n📝 Creating GitHub issues for findings...")

        for result in results:
            if result["status"] != "success":
                continue

            if result.get("findings_count", 0) == 0:
                continue

            repo = result["repo"]
            findings = result["findings_count"]

            # Create issue via GitHub API
            try:
                import requests
            except ImportError:
                print("⚠️  requests required for issue creation. Install with: pip install requests")
                continue

            headers = {}
            if self.github_token:
                headers["Authorization"] = f"Bearer {self.github_token}"

            owner, repo_name = repo.split("/")

            issue_data = {
                "title": f"🔐 Security Vulnerabilities Found ({findings})",
                "body": f"Strix security scan found {findings} vulnerability/vulnerabilities.\n\n"
                        f"See scan results in: {result.get('output_dir')}",
                "labels": ["security", "automated"],
            }

            try:
                url = f"https://api.github.com/repos/{owner}/{repo_name}/issues"
                resp = requests.post(
                    url,
                    json=issue_data,
                    headers=headers,
                    timeout=10,
                )
                if resp.status_code == 201:
                    print(f"   ✓ Created issue for {repo}")
                else:
                    print(f"   ✗ Failed to create issue for {repo}: {resp.status_code}")
            except Exception as e:
                print(f"   ✗ Error creating issue for {repo}: {e}")

    def generate_summary_report(self, results: List[dict]) -> dict:
        """Generate a summary report of all tests."""
        report = {
            "total_repos": len(results),
            "successful_scans": sum(1 for r in results if r["status"] == "success"),
            "failed_scans": sum(1 for r in results if r["status"] in ("error", "timeout")),
            "total_findings": sum(r.get("findings_count", 0) for r in results),
            "repos_with_issues": [
                r for r in results
                if r["status"] == "success" and r.get("findings_count", 0) > 0
            ],
            "results": results,
        }
        return report

    def save_report(self, report: dict, output_path: str = "security-scan-report.json"):
        """Save report to file."""
        with open(output_path, "w") as f:
            json.dump(report, f, indent=2)
        print(f"📄 Report saved to: {output_path}")


def main():
    """CLI entry point."""
    import argparse

    parser = argparse.ArgumentParser(
        description="Multi-agent GitHub repo security testing with Strix"
    )
    parser.add_argument("--org", help="GitHub organization to scan")
    parser.add_argument("--user", help="GitHub user to scan")
    parser.add_argument("--repos", nargs="+", help="Specific repos to test (owner/repo format)")
    parser.add_argument("--workers", type=int, default=3, help="Number of parallel agents")
    parser.add_argument("--output-dir", default="./github-security-scans")
    parser.add_argument("--create-issues", action="store_true", help="Create GitHub issues for findings")
    parser.add_argument("--language", help="Filter repos by language")

    args = parser.parse_args()

    tester = GitHubMultiAgentTester(max_workers=args.workers)

    repos = []

    if args.org:
        repos = tester.discover_org_repos(args.org, language=args.language)
    elif args.user:
        repos = tester.discover_user_repos(args.user)
    elif args.repos:
        for repo_str in args.repos:
            owner, repo_name = repo_str.split("/")
            repos.append(
                GithubRepo(owner=owner, repo=repo_name, url=f"https://github.com/{repo_str}")
            )

    if not repos:
        print("No repositories found or specified")
        sys.exit(1)

    results = tester.batch_test_repos(repos, args.output_dir)

    report = tester.generate_summary_report(results)

    print("\n" + "=" * 70)
    print("SECURITY SCAN SUMMARY")
    print("=" * 70)
    print(f"Total Repositories: {report['total_repos']}")
    print(f"Successful Scans: {report['successful_scans']}")
    print(f"Failed Scans: {report['failed_scans']}")
    print(f"Total Findings: {report['total_findings']}")

    if report["repos_with_issues"]:
        print(f"\n⚠️  Repos with vulnerabilities:")
        for repo_result in report["repos_with_issues"]:
            print(f"   - {repo_result['repo']}: {repo_result.get('findings_count')} findings")

    tester.save_report(report, f"{args.output_dir}/summary-report.json")

    if args.create_issues and report["repos_with_issues"]:
        tester.create_github_issues(results)

    sys.exit(0)


if __name__ == "__main__":
    main()
