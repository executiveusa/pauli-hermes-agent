use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionClass {
    SafeRead,
    SafeWrite,
    SensitiveWrite,
    IrreversibleAction,
}

pub fn classify_action(action: &str) -> ActionClass {
    let a = action.to_lowercase();
    if ["delete", "drop", "destroy", "wipe"].iter().any(|x| a.contains(x)) {
        return ActionClass::IrreversibleAction;
    }
    if ["publish", "push", "merge", "spend", "budget"].iter().any(|x| a.contains(x)) {
        return ActionClass::SensitiveWrite;
    }
    if ["create", "write", "update", "edit"].iter().any(|x| a.contains(x)) {
        return ActionClass::SafeWrite;
    }
    ActionClass::SafeRead
}

pub fn requires_approval(action: &str) -> bool {
    matches!(
        classify_action(action),
        ActionClass::SensitiveWrite | ActionClass::IrreversibleAction
    )
}
