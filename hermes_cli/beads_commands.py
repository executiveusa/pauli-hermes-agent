"""
Hermes CLI Beads slash command integration.

Provides `/bd` command for CLI users to interact with Beads task tracking.

Usage in CLI:
  /bd ready                  - List ready tasks
  /bd create "Task"          - Create new task
  /bd show bd-a1b2          - Show task details
  /bd claim bd-a1b2         - Claim task
  /bd close bd-a1b2 "Fixed" - Close task
"""

from typing import Optional, List
from agent.beads_integration import BeadsTracker
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich.markdown import Markdown

console = Console()


class BeadsCliHandler:
    """Handler for /bd slash commands in Hermes CLI."""

    def __init__(self):
        self.tracker = BeadsTracker()

    def handle_command(self, args: List[str]) -> str:
        """
        Handle /bd command with subcommands.
        
        Args:
            args: Command arguments (e.g., ["ready"], ["create", "Task title"])
            
        Returns:
            Formatted output string
        """
        if not args:
            return self._format_help()

        subcommand = args[0].lower()

        if subcommand == "ready":
            return self._handle_ready()
        elif subcommand == "create":
            return self._handle_create(args[1:])
        elif subcommand == "show":
            return self._handle_show(args[1:])
        elif subcommand == "claim":
            return self._handle_claim(args[1:])
        elif subcommand == "close":
            return self._handle_close(args[1:])
        elif subcommand == "update":
            return self._handle_update(args[1:])
        elif subcommand == "list":
            return self._handle_list(args[1:])
        elif subcommand == "stats":
            return self._handle_stats()
        elif subcommand == "help":
            return self._format_help()
        else:
            return f"Unknown subcommand: {subcommand}\n\n{self._format_help()}"

    def _handle_ready(self) -> str:
        """List ready tasks."""
        try:
            tasks = self.tracker.list_ready()
            
            if not tasks:
                return "✅ All tasks are blocked! No ready tasks at this moment."
            
            # Create table
            table = Table(title=f"Ready Tasks ({len(tasks)})", show_header=True)
            table.add_column("ID", style="cyan")
            table.add_column("Title", style="white")
            table.add_column("Priority", style="yellow")
            table.add_column("Type", style="magenta")
            
            for task in tasks:
                priority_name = ["🔴 Critical", "🟠 High", "🟡 Medium", "🟢 Low"][task.priority]
                table.add_row(task.id, task.title[:50], priority_name, task.type)
            
            console.print(table)
            return f"\n📊 {len(tasks)} ready task(s) available"
        except Exception as e:
            return f"❌ Error listing ready tasks: {str(e)}"

    def _handle_create(self, args: List[str]) -> str:
        """Create new task: /bd create "Title" [--priority 1] [--type task]"""
        if not args:
            return "❌ Usage: /bd create \"Title\" [--priority <0-3>] [--type <type>]"
        
        title = args[0]
        priority = 2
        task_type = "task"
        
        # Parse optional args
        i = 1
        while i < len(args):
            if args[i] == "--priority" and i + 1 < len(args):
                priority = int(args[i + 1])
                i += 2
            elif args[i] == "--type" and i + 1 < len(args):
                task_type = args[i + 1]
                i += 2
            else:
                i += 1
        
        try:
            task = self.tracker.create_task(
                title=title,
                priority=priority,
                type=task_type,
            )
            
            if task:
                return f"✅ Created task: {task.id}\n   Title: {task.title}\n   Priority: {priority}\n   Type: {task_type}"
            else:
                return "❌ Failed to create task"
        except Exception as e:
            return f"❌ Error creating task: {str(e)}"

    def _handle_show(self, args: List[str]) -> str:
        """Show task details: /bd show bd-a1b2"""
        if not args:
            return "❌ Usage: /bd show <task_id>"
        
        task_id = args[0]
        
        try:
            task = self.tracker.get_task(task_id)
            
            if not task:
                return f"❌ Task {task_id} not found"
            
            # Format task details
            output = f"""
╔════════════════════════════════════════════════════════════╗
║ Task: {task.id}
╠════════════════════════════════════════════════════════════╣
║ Title:      {task.title}
║ Status:     {task.status}
║ Priority:   {["Critical", "High", "Medium", "Low"][task.priority]}
║ Type:       {task.type}
║ Assignee:   {task.assignee or "Unassigned"}
║ Created:    {task.created_at or "Unknown"}
║ Updated:    {task.updated_at or "Unknown"}
║ Blockers:   {len(task.blockers or [])} blocker(s)
╚════════════════════════════════════════════════════════════╝
"""
            
            if task.description:
                output += f"\n📝 Description:\n{task.description}\n"
            
            if task.design:
                output += f"\n🏗️  Design Notes:\n{task.design}\n"
            
            if task.acceptance:
                output += f"\n✓ Acceptance Criteria:\n{task.acceptance}\n"
            
            if task.blockers:
                output += f"\n🚫 Blockers:\n" + "\n".join(f"  - {b}" for b in task.blockers) + "\n"
            
            return output
        except Exception as e:
            return f"❌ Error: {str(e)}"

    def _handle_claim(self, args: List[str]) -> str:
        """Claim task: /bd claim bd-a1b2"""
        if not args:
            return "❌ Usage: /bd claim <task_id>"
        
        task_id = args[0]
        
        try:
            success = self.tracker.claim_task(task_id)
            
            if success:
                task = self.tracker.get_task(task_id)
                return f"✅ Claimed task {task_id}\n   Status: {task.status}\n   Assignee: {task.assignee}"
            else:
                return f"❌ Failed to claim task {task_id}"
        except Exception as e:
            return f"❌ Error: {str(e)}"

    def _handle_close(self, args: List[str]) -> str:
        """Close task: /bd close bd-a1b2 "Message" """
        if not args:
            return "❌ Usage: /bd close <task_id> [message]"
        
        task_id = args[0]
        message = args[1] if len(args) > 1 else "Done"
        
        try:
            success = self.tracker.close_task(task_id, message)
            
            if success:
                return f"✅ Closed task {task_id}\n   Message: {message}"
            else:
                return f"❌ Failed to close task {task_id}"
        except Exception as e:
            return f"❌ Error: {str(e)}"

    def _handle_update(self, args: List[str]) -> str:
        """Update task: /bd update bd-a1b2 --notes "Progress" --priority 1"""
        if not args:
            return "❌ Usage: /bd update <task_id> [--notes <text>] [--priority <0-3>] [--status <status>]"
        
        task_id = args[0]
        notes = None
        priority = None
        status = None
        
        # Parse optional args
        i = 1
        while i < len(args):
            if args[i] == "--notes" and i + 1 < len(args):
                notes = args[i + 1]
                i += 2
            elif args[i] == "--priority" and i + 1 < len(args):
                priority = int(args[i + 1])
                i += 2
            elif args[i] == "--status" and i + 1 < len(args):
                status = args[i + 1]
                i += 2
            else:
                i += 1
        
        try:
            success = self.tracker.update_task(
                task_id=task_id,
                notes=notes,
                priority=priority,
                status=status,
            )
            
            if success:
                return f"✅ Updated task {task_id}"
            else:
                return f"❌ Failed to update task {task_id}"
        except Exception as e:
            return f"❌ Error: {str(e)}"

    def _handle_list(self, args: List[str]) -> str:
        """List tasks: /bd list [--status in_progress] [--type bug]"""
        status = None
        task_type = None
        priority = None
        
        # Parse optional args
        i = 0
        while i < len(args):
            if args[i] == "--status" and i + 1 < len(args):
                status = args[i + 1]
                i += 2
            elif args[i] == "--type" and i + 1 < len(args):
                task_type = args[i + 1]
                i += 2
            elif args[i] == "--priority" and i + 1 < len(args):
                priority = int(args[i + 1])
                i += 2
            else:
                i += 1
        
        try:
            tasks = self.tracker.list_all(status=status, type=task_type, priority=priority)
            
            if not tasks:
                return "❌ No tasks found matching criteria"
            
            # Create table
            table = Table(title=f"Tasks ({len(tasks)})", show_header=True)
            table.add_column("ID", style="cyan")
            table.add_column("Title", style="white")
            table.add_column("Status", style="green")
            table.add_column("Priority", style="yellow")
            
            for task in tasks:
                priority_name = ["🔴", "🟠", "🟡", "🟢"][task.priority]
                table.add_row(task.id, task.title[:40], task.status, priority_name)
            
            console.print(table)
            return f"\n📊 {len(tasks)} task(s) found"
        except Exception as e:
            return f"❌ Error: {str(e)}"

    def _handle_stats(self) -> str:
        """Show statistics."""
        try:
            all_tasks = self.tracker.list_all()
            ready_tasks = self.tracker.list_ready()
            
            by_status = {
                "not_started": len([t for t in all_tasks if t.status == "not_started"]),
                "in_progress": len([t for t in all_tasks if t.status == "in_progress"]),
                "ready_for_review": len([t for t in all_tasks if t.status == "ready_for_review"]),
                "closed": len([t for t in all_tasks if t.status == "closed"]),
            }
            
            output = f"""
╔════════════════════════════════════════════════════════════╗
║  Beads Task Tracker Statistics
╠════════════════════════════════════════════════════════════╣
║  Total Tasks:           {len(all_tasks)}
║  Ready Count:           {len(ready_tasks)}
║
║  By Status:
║    • Not Started:       {by_status["not_started"]}
║    • In Progress:       {by_status["in_progress"]}
║    • Ready for Review:  {by_status["ready_for_review"]}
║    • Closed:            {by_status["closed"]}
╚════════════════════════════════════════════════════════════╝
"""
            return output
        except Exception as e:
            return f"❌ Error: {str(e)}"

    def _format_help(self) -> str:
        """Format help text."""
        return """
╔════════════════════════════════════════════════════════════╗
║  Beads Task Tracker - CLI Commands
╠════════════════════════════════════════════════════════════╣
║  /bd ready                   List tasks with no blockers
║  /bd create "Title"          Create new task
║  /bd show bd-a1b2           Show task details
║  /bd claim bd-a1b2          Claim task (you become assignee)
║  /bd close bd-a1b2 "Fixed"  Close task with message
║  /bd update bd-a1b2 --notes "text" --priority 1
║  /bd list [--status in_progress] [--type bug]
║  /bd stats                   Show statistics
║  /bd help                    Show this help
╠════════════════════════════════════════════════════════════╣
║ Documentation: https://github.com/gastownhall/beads
║ Agent Instructions: See AGENT_INSTRUCTIONS.md
╚════════════════════════════════════════════════════════════╝

Examples:
  /bd create "Fix auth bug" --priority 1 --type bug
  /bd claim bd-hermes.4.2
  /bd close bd-hermes.4.2 "Fixed and tested - all tests passing"
  /bd list --status in_progress
  /bd show bd-hermes.4.2
"""


# Export for use in cli.py
def register_beads_commands():
    """Register /bd command in Hermes CLI."""
    return {
        "name": "bd",
        "description": "Beads task tracking - persistent agent memory",
        "handler": BeadsCliHandler,
    }
