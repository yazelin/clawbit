use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEvent {
    pub session_id: String,
    pub event: HookEventType,
    pub status: SessionStatus,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub user_prompt: Option<String>,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub tool_use_id: Option<String>,
    #[serde(default)]
    pub tool_input: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub permission_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HookEventType {
    UserPromptSubmit,
    SessionStart,
    PreToolUse,
    PostToolUse,
    PermissionRequest,
    PreCompact,
    Stop,
    SubagentStop,
    SessionEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Processing,
    WaitingForInput,
    RunningTool,
    Compacting,
    Ended,
    Unknown,
}
