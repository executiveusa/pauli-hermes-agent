from __future__ import annotations

from enum import Enum


class ActionClass(str, Enum):
    SAFE_READ = "safe_read"
    SAFE_WRITE = "safe_write"
    SENSITIVE_WRITE = "sensitive_write"
    IRREVERSIBLE_ACTION = "irreversible_action"


def classify_action(action: str) -> ActionClass:
    a = action.lower()
    if any(x in a for x in ("delete", "drop", "revoke", "destroy", "wipe")):
        return ActionClass.IRREVERSIBLE_ACTION
    if any(x in a for x in ("rotate_secret", "set_env", "publish", "push", "merge", "budget_override")):
        return ActionClass.SENSITIVE_WRITE
    if any(x in a for x in ("write", "create", "update", "edit", "apply_patch", "commit")):
        return ActionClass.SAFE_WRITE
    return ActionClass.SAFE_READ


def requires_approval(action: str) -> bool:
    return classify_action(action) in {
        ActionClass.SENSITIVE_WRITE,
        ActionClass.IRREVERSIBLE_ACTION,
    }
