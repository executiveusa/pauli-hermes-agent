"""
Beads REST API endpoints for Hermes gateway.

Exposes Beads task management as REST API for dashboard and external clients.

Endpoints:
  GET  /v1/beads/ready - List ready tasks
  GET  /v1/beads - List all tasks with filters
  GET  /v1/beads/:id - Get task details
  POST /v1/beads - Create new task
  PUT  /v1/beads/:id - Update task
  POST /v1/beads/:id/claim - Claim task
  POST /v1/beads/:id/close - Close task
  GET  /v1/beads/stats - Summary statistics
"""

from fastapi import APIRouter, Query, HTTPException, Body, Depends
from typing import Optional, List
from datetime import datetime
from agent.beads_integration import BeadsTracker, BeadsTask
import json

router = APIRouter(prefix="/v1/beads", tags=["beads"])

# Dependency: Get tracker instance
def get_tracker() -> BeadsTracker:
    """Return BeadsTracker instance."""
    return BeadsTracker()


@router.get("/ready", response_model=List[dict])
async def get_ready_tasks(
    assignee: Optional[str] = Query(None, description="Filter by assignee"),
    tracker: BeadsTracker = Depends(get_tracker),
):
    """
    List all tasks with no open blockers (ready to work on).
    
    Query Parameters:
    - assignee: Filter by assignee email
    
    Returns:
    - List of ready tasks sorted by priority
    """
    try:
        tasks = tracker.list_ready(assignee=assignee)
        return [task.to_dict() for task in tasks]
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to list ready tasks: {str(e)}")


@router.get("", response_model=List[dict])
async def list_tasks(
    status: Optional[str] = Query(None, description="Filter by status"),
    type: Optional[str] = Query(None, description="Filter by type (task|bug|feature|chore|epic)"),
    priority: Optional[int] = Query(None, description="Filter by priority (0-3)"),
    limit: Optional[int] = Query(50, description="Max results"),
    tracker: BeadsTracker = Depends(get_tracker),
):
    """
    List all tasks with optional filters.
    
    Query Parameters:
    - status: not_started|in_progress|ready_for_review|closed
    - type: task|bug|feature|chore|epic
    - priority: 0 (critical), 1 (high), 2 (medium), 3 (low)
    - limit: Max number of results
    
    Returns:
    - List of tasks
    """
    try:
        tasks = tracker.list_all(status=status, type=type, priority=priority)
        return [task.to_dict() for task in tasks[:limit]]
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to list tasks: {str(e)}")


@router.get("/{task_id}", response_model=dict)
async def get_task(
    task_id: str,
    tracker: BeadsTracker = Depends(get_tracker),
):
    """
    Get a single task by ID.
    
    Path Parameters:
    - task_id: Beads task ID (e.g., "bd-a1b2")
    
    Returns:
    - Task object with full details
    """
    try:
        task = tracker.get_task(task_id)
        if not task:
            raise HTTPException(status_code=404, detail=f"Task {task_id} not found")
        return task.to_dict()
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to get task: {str(e)}")


@router.post("", response_model=dict, status_code=201)
async def create_task(
    title: str = Body(..., description="Task title"),
    priority: int = Body(2, description="Priority: 0=critical, 1=high, 2=medium, 3=low"),
    type: str = Body("task", description="Type: task|bug|feature|chore|epic"),
    parent: Optional[str] = Body(None, description="Parent epic/task ID"),
    assignee: Optional[str] = Body(None, description="Assignee email"),
    description: Optional[str] = Body(None, description="Full description"),
    design: Optional[str] = Body(None, description="Design/architecture notes"),
    acceptance: Optional[str] = Body(None, description="Acceptance criteria"),
    tracker: BeadsTracker = Depends(get_tracker),
):
    """
    Create a new task.
    
    Request Body:
    ```json
    {
      "title": "Implement feature X",
      "priority": 1,
      "type": "feature",
      "parent": "bd-abc123",
      "assignee": "agent@hermes.local",
      "description": "Detailed description",
      "design": "Architecture approach",
      "acceptance": "Acceptance criteria"
    }
    ```
    
    Returns:
    - Created task object with ID
    """
    try:
        task = tracker.create_task(
            title=title,
            priority=priority,
            type=type,
            parent=parent,
            assignee=assignee,
            description=description,
            design=design,
            acceptance=acceptance,
        )
        if not task:
            raise HTTPException(status_code=400, detail="Failed to create task")
        return task.to_dict()
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to create task: {str(e)}")


@router.put("/{task_id}", response_model=dict)
async def update_task(
    task_id: str,
    status: Optional[str] = Body(None, description="New status"),
    notes: Optional[str] = Body(None, description="Append notes"),
    priority: Optional[int] = Body(None, description="Update priority"),
    labels: Optional[List[str]] = Body(None, description="Add labels"),
    tracker: BeadsTracker = Depends(get_tracker),
):
    """
    Update a task.
    
    Path Parameters:
    - task_id: Beads task ID
    
    Request Body:
    ```json
    {
      "status": "in_progress",
      "notes": "Started implementation",
      "priority": 1,
      "labels": ["api", "critical"]
    }
    ```
    
    Returns:
    - Updated task object
    """
    try:
        success = tracker.update_task(
            task_id=task_id,
            status=status,
            notes=notes,
            priority=priority,
            labels=labels,
        )
        if not success:
            raise HTTPException(status_code=400, detail="Failed to update task")
        
        task = tracker.get_task(task_id)
        if not task:
            raise HTTPException(status_code=404, detail="Task not found after update")
        return task.to_dict()
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to update task: {str(e)}")


@router.post("/{task_id}/claim", response_model=dict)
async def claim_task(
    task_id: str,
    tracker: BeadsTracker = Depends(get_tracker),
):
    """
    Claim a task (atomically set assignee + in_progress status).
    
    Path Parameters:
    - task_id: Beads task ID
    
    Returns:
    - Claimed task object
    """
    try:
        success = tracker.claim_task(task_id)
        if not success:
            raise HTTPException(status_code=400, detail="Failed to claim task")
        
        task = tracker.get_task(task_id)
        if not task:
            raise HTTPException(status_code=404, detail="Task not found")
        return task.to_dict()
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to claim task: {str(e)}")


@router.post("/{task_id}/close", response_model=dict)
async def close_task(
    task_id: str,
    message: str = Body("Done", description="Closing message"),
    tracker: BeadsTracker = Depends(get_tracker),
):
    """
    Close a task.
    
    Path Parameters:
    - task_id: Beads task ID
    
    Request Body:
    ```json
    {
      "message": "Fixed and tested"
    }
    ```
    
    Returns:
    - Closed task object
    """
    try:
        success = tracker.close_task(task_id, message)
        if not success:
            raise HTTPException(status_code=400, detail="Failed to close task")
        
        task = tracker.get_task(task_id)
        if not task:
            raise HTTPException(status_code=404, detail="Task not found")
        return task.to_dict()
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to close task: {str(e)}")


@router.post("/{child_id}/depends-on/{parent_id}", response_model=dict)
async def add_dependency(
    child_id: str,
    parent_id: str,
    relation: str = Query("blocks", description="Relation type: blocks|relates-to|duplicates|supersedes"),
    tracker: BeadsTracker = Depends(get_tracker),
):
    """
    Add a dependency between tasks.
    
    Path Parameters:
    - child_id: Task ID that is blocked
    - parent_id: Task ID that blocks
    
    Query Parameters:
    - relation: Type of relation (blocks, relates-to, duplicates, supersedes)
    
    Returns:
    - Confirmation message
    """
    try:
        success = tracker.add_dependency(child_id, parent_id, relation=relation)
        if not success:
            raise HTTPException(status_code=400, detail="Failed to add dependency")
        
        return {
            "status": "ok",
            "child_id": child_id,
            "parent_id": parent_id,
            "relation": relation,
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to add dependency: {str(e)}")


@router.get("/stats", response_model=dict)
async def get_stats(
    tracker: BeadsTracker = Depends(get_tracker),
):
    """
    Get summary statistics about tasks.
    
    Returns:
    - Task counts by status, priority, type
    - Average time-to-close
    - Ready task count
    """
    try:
        all_tasks = tracker.list_all()
        ready_tasks = tracker.list_ready()
        
        stats = {
            "total_tasks": len(all_tasks),
            "by_status": {
                "not_started": len([t for t in all_tasks if t.status == "not_started"]),
                "in_progress": len([t for t in all_tasks if t.status == "in_progress"]),
                "ready_for_review": len([t for t in all_tasks if t.status == "ready_for_review"]),
                "closed": len([t for t in all_tasks if t.status == "closed"]),
            },
            "by_priority": {
                "critical": len([t for t in all_tasks if t.priority == 0]),
                "high": len([t for t in all_tasks if t.priority == 1]),
                "medium": len([t for t in all_tasks if t.priority == 2]),
                "low": len([t for t in all_tasks if t.priority == 3]),
            },
            "by_type": {
                "task": len([t for t in all_tasks if t.type == "task"]),
                "bug": len([t for t in all_tasks if t.type == "bug"]),
                "feature": len([t for t in all_tasks if t.type == "feature"]),
                "chore": len([t for t in all_tasks if t.type == "chore"]),
                "epic": len([t for t in all_tasks if t.type == "epic"]),
            },
            "ready_count": len(ready_tasks),
            "timestamp": datetime.utcnow().isoformat(),
        }
        
        return stats
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to get stats: {str(e)}")
