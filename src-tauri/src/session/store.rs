use crate::models::{Emotion, HookEvent, HookEventType, TaskState};
use crate::state::StateMachine;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
pub struct SessionData {
    pub session_id: String,
    pub task_state: TaskState,
    pub emotion: Emotion,
    pub cwd: Option<String>,
    pub tool_calls: u32,
    pub turns: u32,
    #[serde(skip)]
    pub last_activity: Instant,
    pub position_index: usize,
}

pub struct SessionStore {
    sessions: HashMap<String, SessionData>,
    next_position: usize,
}

impl SessionStore {
    pub fn new() -> Self {
        Self { sessions: HashMap::new(), next_position: 0 }
    }

    pub fn handle_event(&mut self, event: &HookEvent) -> &SessionData {
        let new_state = StateMachine::next_state(&event.event);

        let session = self.sessions.entry(event.session_id.clone()).or_insert_with(|| {
            let pos = self.next_position;
            self.next_position += 1;
            SessionData {
                session_id: event.session_id.clone(),
                task_state: TaskState::Idle,
                emotion: Emotion::Neutral,
                cwd: event.cwd.clone(),
                tool_calls: 0,
                turns: 0,
                last_activity: Instant::now(),
                position_index: pos,
            }
        });

        session.task_state = new_state;
        session.last_activity = Instant::now();

        if event.tool.is_some() { session.tool_calls += 1; }
        if event.event == HookEventType::UserPromptSubmit { session.turns += 1; }
        if event.cwd.is_some() { session.cwd.clone_from(&event.cwd); }

        session
    }

    pub fn get_sessions(&self) -> Vec<&SessionData> {
        self.sessions.values().collect()
    }

    pub fn check_sleepers(&mut self) {
        let timeout = std::time::Duration::from_secs(300);
        for session in self.sessions.values_mut() {
            if session.last_activity.elapsed() > timeout
                && session.task_state != TaskState::Sleeping
                && session.task_state != TaskState::Idle
            {
                session.task_state = TaskState::Sleeping;
            }
        }
    }

    pub fn remove_ended(&mut self) {
        self.sessions.retain(|_, s| {
            s.task_state != TaskState::Idle || s.last_activity.elapsed() < std::time::Duration::from_secs(3600)
        });
    }

    pub fn update_emotion(&mut self, session_id: &str, emotion: Emotion) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.emotion = emotion;
        }
    }
}
