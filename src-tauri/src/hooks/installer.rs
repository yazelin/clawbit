use serde_json::{json, Value};
use std::path::PathBuf;

const HOOK_EVENTS: &[&str] = &[
    "UserPromptSubmit",
    "SessionStart",
    "PreToolUse",
    "PostToolUse",
    "PermissionRequest",
    "PreCompact",
    "Stop",
    "SubagentStop",
    "SessionEnd",
];

const EVENTS_WITH_MATCHER: &[&str] = &["PreToolUse", "PostToolUse", "PermissionRequest"];

fn claude_settings_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".claude")
        .join("settings.json")
}

fn hook_binary_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".claude")
        .join("hooks")
        .join(if cfg!(windows) {
            "clawbit-hook.exe"
        } else {
            "clawbit-hook"
        })
}

fn hook_entry(command: &str) -> Value {
    json!({"type": "command", "command": command})
}

pub fn install() -> Result<(), String> {
    let hook_bin = hook_binary_path();
    let hooks_dir = hook_bin.parent().unwrap();
    std::fs::create_dir_all(hooks_dir)
        .map_err(|e| format!("Failed to create hooks dir: {}", e))?;

    let command = hook_bin.to_string_lossy().to_string();
    let settings_path = claude_settings_path();

    let mut settings: Value = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings: {}", e))?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    if settings.get("hooks").is_none() {
        settings["hooks"] = json!({});
    }

    for event in HOOK_EVENTS {
        let already_installed = settings["hooks"]
            .get(*event)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().any(|entry| {
                    entry["hooks"]
                        .as_array()
                        .map(|hooks| {
                            hooks
                                .iter()
                                .any(|h| h["command"].as_str() == Some(&command))
                        })
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        if already_installed {
            continue;
        }

        let hook_config = if *event == "PreCompact" {
            json!([
                {"matcher": "auto", "hooks": [hook_entry(&command)]},
                {"matcher": "manual", "hooks": [hook_entry(&command)]}
            ])
        } else if EVENTS_WITH_MATCHER.contains(event) {
            json!([{"matcher": ".*", "hooks": [hook_entry(&command)]}])
        } else {
            json!([{"hooks": [hook_entry(&command)]}])
        };

        settings["hooks"][*event] = hook_config;
    }

    let output = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&settings_path, output).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let settings_path = claude_settings_path();
    if !settings_path.exists() {
        return Ok(());
    }

    let command = hook_binary_path().to_string_lossy().to_string();
    let content = std::fs::read_to_string(&settings_path).map_err(|e| e.to_string())?;
    let mut settings: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    if let Some(hooks) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        let mut empty_events = Vec::new();

        for event in HOOK_EVENTS {
            if let Some(entries) = hooks.get_mut(*event).and_then(|v| v.as_array_mut()) {
                entries.retain(|entry| {
                    !entry["hooks"]
                        .as_array()
                        .map(|hooks| {
                            hooks
                                .iter()
                                .any(|h| h["command"].as_str() == Some(&command))
                        })
                        .unwrap_or(false)
                });
                if entries.is_empty() {
                    empty_events.push(event.to_string());
                }
            }
        }

        for event in empty_events {
            hooks.remove(&event);
        }
    }

    let output = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&settings_path, output).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(hook_binary_path());
    Ok(())
}
