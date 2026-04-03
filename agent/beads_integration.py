"""
Beads integration for Hermes Agent - persistent task tracking system.

This module provides a Python wrapper around the `bd` CLI for task tracking,
integrated with the Hermes agent system and API gateway.

Usage:
    from agent.beads_integration import BeadsTracker
    tracker = BeadsTracker()
    
    # Create an epic
    epic = tracker.create_epic("Hermes 3.0 Production Build", priority=0)
    
    # Create a task
    task = tracker.create_task(
        "Implement Supabase integration",
        priority=1,
        parent=epic.id,
        assignee="agent@hermes.local"
    )
    
    # Claim task
    tracker.claim_task(task.id)
    
    # Update with notes
    tracker.update_task(task.id, notes="Completed schema migration")
    
    # Close task
    tracker.close_task(task.id, "Supabase fully integrated")
"""

import json
import os
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Optional, List, Dict, Any

import asyncio
import aiofiles


@dataclass
class BeadsTask:
    """Represents a single task in Beads."""
    id: str
    title: str
    status: str
    priority: int
    type: str
    assignee: Optional[str] = None
    description: Optional[str] = None
    design: Optional[str] = None
    acceptance: Optional[str] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None
    closed_at: Optional[str] = None
    blockers: Optional[List[str]] = None
    metadata: Optional[Dict[str, Any]] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert task to dictionary for JSON serialization."""
        return {
            "id": self.id,
            "title": self.title,
            "status": self.status,
            "priority": self.priority,
            "type": self.type,
            "assignee": self.assignee,
            "description": self.description,
            "design": self.design,
            "acceptance": self.acceptance,
            "created_at": self.created_at,
            "updated_at": self.updated_at,
            "closed_at": self.closed_at,
            "blockers": self.blockers or [],
            "metadata": self.metadata or {},
        }


class BeadsTracker:
    """
    Python wrapper around Beads CLI for Hermes task tracking.
    
    Provides methods to create, update, query, and manage tasks in the Beads database.
    Integrates with Hermes API and dashboard.
    """

    def __init__(self, workspace_root: Optional[str] = None, agent_id: str = "agent@hermes.local"):
        """
        Initialize Beads tracker.
        
        Args:
            workspace_root: Path to workspace (auto-detected if None)
            agent_id: Agent identifier for assignee field
        """
        self.workspace_root = workspace_root or self._find_workspace()
        self.beads_dir = Path(self.workspace_root) / ".beads"
        self.agent_id = agent_id
        self._ensure_initialized()

    def _find_workspace(self) -> str:
        """Find workspace root by looking for .git or hermes markers."""
        current = Path.cwd()
        for _ in range(10):  # Search up to 10 levels
            if (current / ".git").exists() or (current / "hermes_cli").exists():
                return str(current)
            current = current.parent
        # Fallback
        return str(Path.cwd())

    def _ensure_initialized(self) -> None:
        """Ensure Beads is initialized in workspace."""
        if not self.beads_dir.exists():
            self._run_command(["bd", "init", "--stealth", "--quiet"])

    def _run_command(self, cmd: List[str], check: bool = True) -> str:
        """
        Run a Beads CLI command and return output.
        
        Args:
            cmd: Command list (e.g., ["bd", "ready", "--json"])
            check: Raise CalledProcessError if return code != 0
            
        Returns:
            Command stdout as string
        """
        try:
            result = subprocess.run(
                cmd,
                cwd=self.workspace_root,
                capture_output=True,
                text=True,
                check=check,
            )
            return result.stdout.strip()
        except subprocess.CalledProcessError as e:
            # Log error but don't fail loudly
            print(f"Beads command failed: {' '.join(cmd)}")
            print(f"Error: {e.stderr}")
            if check:
                raise
            return ""

    def create_task(
        self,
        title: str,
        priority: int = 2,
        type: str = "task",
        parent: Optional[str] = None,
        assignee: Optional[str] = None,
        description: Optional[str] = None,
        design: Optional[str] = None,
        acceptance: Optional[str] = None,
    ) -> Optional[BeadsTask]:
        """
        Create a new task in Beads.
        
        Args:
            title: Task title (1-liner)
            priority: 0=critical, 1=high, 2=medium, 3=low
            type: bug|task|feature|chore
            parent: Parent epic/task ID for hierarchical organization
            assignee: Assignee email (defaults to agent_id)
            description: Full task description
            design: Architecture/design notes
            acceptance: Acceptance criteria (markdown)
            
        Returns:
            BeadsTask object or None if creation failed
        """
        cmd = ["bd", "create", title, "-p", str(priority), "-t", type]
        
        if parent:
            cmd.extend(["--parent", parent])
        
        assignee = assignee or self.agent_id
        cmd.extend(["--assignee", assignee])
        
        if description:
            # Pipe description via stdin (avoids shell escaping issues)
            result = subprocess.run(
                cmd + ["--description", description],
                cwd=self.workspace_root,
                capture_output=True,
                text=True,
                check=False,
            )
        else:
            result = subprocess.run(
                cmd,
                cwd=self.workspace_root,
                capture_output=True,
                text=True,
                check=False,
            )
        
        if result.returncode != 0:
            print(f"Failed to create task: {result.stderr}")
            return None
        
        # Extract task ID from output (e.g., "Created bd-a1b2")
        output = result.stdout.strip()
        if "bd-" in output:
            task_id = output.split("bd-")[1].split()[0]
            task_id = f"bd-{task_id}"
            
            # Update with design and acceptance if provided
            if design:
                self._run_command(["bd", "update", task_id, "--design", design], check=False)
            if acceptance:
                self._run_command(["bd", "update", task_id, "--acceptance", acceptance], check=False)
            
            return self.get_task(task_id)
        
        return None

    def create_epic(
        self,
        title: str,
        priority: int = 1,
        assignee: Optional[str] = None,
        description: Optional[str] = None,
    ) -> Optional[BeadsTask]:
        """Create an epic (for organizing related tasks)."""
        return self.create_task(
            title,
            priority=priority,
            type="epic",
            assignee=assignee,
            description=description,
        )

    def get_task(self, task_id: str) -> Optional[BeadsTask]:
        """
        Retrieve a task by ID.
        
        Args:
            task_id: Beads task ID (e.g., "bd-a1b2")
            
        Returns:
            BeadsTask object or None if not found
        """
        output = self._run_command(["bd", "show", task_id, "--json"], check=False)
        if not output:
            return None
        
        try:
            data = json.loads(output)
            return BeadsTask(
                id=data.get("id"),
                title=data.get("title"),
                status=data.get("status", "not_started"),
                priority=data.get("priority", 2),
                type=data.get("type", "task"),
                assignee=data.get("assignee"),
                description=data.get("description"),
                design=data.get("design"),
                acceptance=data.get("acceptance"),
                created_at=data.get("created_at"),
                updated_at=data.get("updated_at"),
                closed_at=data.get("closed_at"),
                blockers=data.get("blockers", []),
                metadata=data.get("metadata", {}),
            )
        except json.JSONDecodeError:
            return None

    def list_ready(self, assignee: Optional[str] = None) -> List[BeadsTask]:
        """
        List all tasks with no open blockers (ready to work on).
        
        Args:
            assignee: Filter by assignee (defaults to current agent)
            
        Returns:
            List of BeadsTask objects
        """
        cmd = ["bd", "ready", "--json"]
        if assignee:
            cmd.extend(["--assignee", assignee])
        
        output = self._run_command(cmd, check=False)
        if not output:
            return []
        
        try:
            tasks_data = json.loads(output)
            tasks = []
            for data in tasks_data:
                task = BeadsTask(
                    id=data.get("id"),
                    title=data.get("title"),
                    status=data.get("status", "not_started"),
                    priority=data.get("priority", 2),
                    type=data.get("type", "task"),
                    assignee=data.get("assignee"),
                    blockers=data.get("blockers", []),
                )
                tasks.append(task)
            return sorted(tasks, key=lambda t: t.priority)  # Sort by priority
        except json.JSONDecodeError:
            return []

    def claim_task(self, task_id: str) -> bool:
        """
        Claim a task (atomically sets assignee + in_progress status).
        
        Args:
            task_id: Beads task ID
            
        Returns:
            True if successful
        """
        output = self._run_command(
            ["bd", "update", task_id, "--claim"],
            check=False
        )
        return "Updated" in output or "claimed" in output

    def update_task(
        self,
        task_id: str,
        status: Optional[str] = None,
        notes: Optional[str] = None,
        priority: Optional[int] = None,
        labels: Optional[List[str]] = None,
    ) -> bool:
        """
        Update task properties.
        
        Args:
            task_id: Beads task ID
            status: New status (not_started|in_progress|ready_for_review)
            notes: Append notes to task history
            priority: Update priority
            labels: Add labels/tags
            
        Returns:
            True if successful
        """
        cmd = ["bd", "update", task_id]
        
        if status:
            cmd.extend(["--status", status])
        if notes:
            cmd.extend(["--notes", notes])
        if priority is not None:
            cmd.extend(["-p", str(priority)])
        if labels:
            for label in labels:
                cmd.extend(["--label", label])
        
        output = self._run_command(cmd, check=False)
        return "Updated" in output

    def close_task(self, task_id: str, message: str = "Done") -> bool:
        """
        Close a task.
        
        Args:
            task_id: Beads task ID
            message: Closing message (e.g., "Fixed", "Won't fix")
            
        Returns:
            True if successful
        """
        output = self._run_command(
            ["bd", "update", task_id, "--close", message],
            check=False
        )
        return "closed" in output.lower()

    def add_dependency(self, child_id: str, parent_id: str, relation: str = "blocks") -> bool:
        """
        Add a dependency between tasks.
        
        Args:
            child_id: Task ID that is blocked
            parent_id: Task ID that blocks
            relation: "blocks" (default) | "relates-to" | "duplicates" | "supersedes"
            
        Returns:
            True if successful
        """
        cmd = ["bd", "dep", "add", child_id, parent_id]
        if relation != "blocks":
            cmd.append(f"--{relation}")
        
        output = self._run_command(cmd, check=False)
        return "Added" in output or "added" in output

    def list_all(
        self,
        status: Optional[str] = None,
        type: Optional[str] = None,
        priority: Optional[int] = None,
    ) -> List[BeadsTask]:
        """
        List all tasks with optional filters.
        
        Args:
            status: Filter by status
            type: Filter by type (task|bug|feature|chore|epic)
            priority: Filter by priority
            
        Returns:
            List of BeadsTask objects
        """
        cmd = ["bd", "list", "--all", "--json"]
        
        if status:
            cmd.extend(["--status", status])
        if type:
            cmd.extend(["--type", type])
        if priority is not None:
            cmd.extend(["--priority", str(priority)])
        
        output = self._run_command(cmd, check=False)
        if not output:
            return []
        
        try:
            tasks_data = json.loads(output)
            tasks = []
            for data in tasks_data:
                task = BeadsTask(
                    id=data.get("id"),
                    title=data.get("title"),
                    status=data.get("status", "not_started"),
                    priority=data.get("priority", 2),
                    type=data.get("type", "task"),
                    assignee=data.get("assignee"),
                    created_at=data.get("created_at"),
                    blockers=data.get("blockers", []),
                )
                tasks.append(task)
            return tasks
        except json.JSONDecodeError:
            return []

    def export_json(self, filepath: str) -> bool:
        """Export all tasks as JSON."""
        try:
            output = self._run_command(["bd", "list", "--all", "--json"], check=False)
            with open(filepath, "w") as f:
                f.write(output)
            return True
        except Exception as e:
            print(f"Failed to export: {e}")
            return False

    def backup(self, backup_path: str) -> bool:
        """Backup Beads database."""
        try:
            self._run_command(["bd", "backup", "init", backup_path])
            self._run_command(["bd", "backup", "sync"])
            return True
        except Exception:
            return False


# Async wrapper for integration with Hermes gateway
class AsyncBeadsTracker:
    """Async wrapper around BeadsTracker for use in gateway/API contexts."""

    def __init__(self, workspace_root: Optional[str] = None, agent_id: str = "agent@hermes.local"):
        self.tracker = BeadsTracker(workspace_root=workspace_root, agent_id=agent_id)

    async def create_task(self, **kwargs) -> Optional[BeadsTask]:
        """Async wrapper for create_task."""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, lambda: self.tracker.create_task(**kwargs))

    async def get_task(self, task_id: str) -> Optional[BeadsTask]:
        """Async wrapper for get_task."""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, lambda: self.tracker.get_task(task_id))

    async def list_ready(self, assignee: Optional[str] = None) -> List[BeadsTask]:
        """Async wrapper for list_ready."""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, lambda: self.tracker.list_ready(assignee))

    async def claim_task(self, task_id: str) -> bool:
        """Async wrapper for claim_task."""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, lambda: self.tracker.claim_task(task_id))

    async def close_task(self, task_id: str, message: str = "Done") -> bool:
        """Async wrapper for close_task."""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(
            None,
            lambda: self.tracker.close_task(task_id, message)
        )
