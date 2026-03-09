use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, Read};

#[derive(Deserialize)]
struct HookInput {
    hook_event_name: String,
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    tool_name: Option<String>,
    #[serde(default)]
    tool_use_id: Option<String>,
    #[serde(default)]
    tool_input: Option<HashMap<String, Value>>,
    #[serde(default)]
    permission_mode: Option<String>,
}

#[derive(Serialize)]
struct HookOutput {
    session_id: String,
    event: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_use_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_input: Option<HashMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permission_mode: Option<String>,
}

fn map_status(event: &str) -> &'static str {
    match event {
        "UserPromptSubmit" => "processing",
        "PreCompact" => "compacting",
        "SessionStart" => "waiting_for_input",
        "SessionEnd" => "ended",
        "PreToolUse" => "running_tool",
        "PostToolUse" => "processing",
        "PermissionRequest" => "waiting_for_input",
        "Stop" => "waiting_for_input",
        "SubagentStop" => "waiting_for_input",
        _ => "unknown",
    }
}

fn main() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        return;
    }

    let hook_input: HookInput = match serde_json::from_str(&input) {
        Ok(v) => v,
        Err(_) => return,
    };

    let session_id = hook_input.session_id.unwrap_or_else(|| "unknown".into());
    let status = map_status(&hook_input.hook_event_name).to_string();

    let output = HookOutput {
        session_id,
        event: hook_input.hook_event_name,
        status,
        cwd: hook_input.cwd,
        user_prompt: hook_input.prompt,
        tool: hook_input.tool_name,
        tool_use_id: hook_input.tool_use_id,
        tool_input: hook_input.tool_input,
        permission_mode: hook_input.permission_mode,
    };

    let json = match serde_json::to_string(&output) {
        Ok(j) => j,
        Err(_) => return,
    };

    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::net::UnixStream;
        if let Ok(mut stream) = UnixStream::connect("/tmp/clawbit.sock") {
            let _ = stream.write_all(json.as_bytes());
        }
    }

    #[cfg(windows)]
    {
        use std::fs::OpenOptions;
        use std::io::Write;
        if let Ok(mut pipe) = OpenOptions::new().write(true).open(r"\\.\pipe\clawbit") {
            let _ = pipe.write_all(json.as_bytes());
        }
    }
}
