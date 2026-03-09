use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Idle,
    Working,
    Sleeping,
    Compacting,
    Waiting,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Emotion {
    Neutral,
    Happy,
    Sad,
    Sob,
}

impl Default for TaskState {
    fn default() -> Self {
        Self::Idle
    }
}

impl Default for Emotion {
    fn default() -> Self {
        Self::Neutral
    }
}
