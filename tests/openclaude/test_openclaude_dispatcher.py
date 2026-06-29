from pathlib import Path


def test_openclaude_wrapper_scripts_exist():
    assert Path("scripts/pauli/openclaude/dispatch.py").exists()
    assert Path("scripts/pauli/openclaude/doctor.py").exists()


def test_openclaude_dispatcher_allows_safe_tasks_and_blocks_dangerous_tasks():
    from pauli.openclaude.dispatcher import OpenClaudeDispatcher

    dispatcher = OpenClaudeDispatcher()

    safe = dispatcher.dispatch("draft a summary of this repository")
    assert safe["allowed"] is True
    assert safe["task_type"] == "safe"

    dangerous = dispatcher.dispatch("delete the auth database")
    assert dangerous["allowed"] is False
    assert dangerous["task_type"] == "destructive"
