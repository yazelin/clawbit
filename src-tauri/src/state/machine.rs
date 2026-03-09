use crate::models::{HookEventType, TaskState};

pub struct StateMachine;

impl StateMachine {
    pub fn next_state(event: &HookEventType) -> TaskState {
        match event {
            HookEventType::UserPromptSubmit => TaskState::Working,
            HookEventType::SessionStart => TaskState::Idle,
            HookEventType::PreToolUse => TaskState::Working,
            HookEventType::PostToolUse => TaskState::Working,
            HookEventType::PermissionRequest => TaskState::Waiting,
            HookEventType::PreCompact => TaskState::Compacting,
            HookEventType::Stop | HookEventType::SubagentStop => TaskState::Idle,
            HookEventType::SessionEnd => TaskState::Idle,
        }
    }
}
