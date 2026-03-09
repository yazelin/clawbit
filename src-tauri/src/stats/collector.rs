use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct SessionStats {
    pub total_turns: u32,
    pub total_tool_calls: u32,
    pub total_errors: u32,
    pub session_start: u64,
    pub tools_used: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AggregateStats {
    pub total_sessions: u32,
    pub total_turns: u32,
    pub total_tool_calls: u32,
}

pub struct StatsCollector {
    session_stats: HashMap<String, SessionStats>,
    aggregate: AggregateStats,
}

impl StatsCollector {
    pub fn new() -> Self {
        Self {
            session_stats: HashMap::new(),
            aggregate: AggregateStats { total_sessions: 0, total_turns: 0, total_tool_calls: 0 },
        }
    }

    pub fn record_turn(&mut self, session_id: &str) {
        let stats = self.get_or_create(session_id);
        stats.total_turns += 1;
        self.aggregate.total_turns += 1;
    }

    pub fn record_tool_call(&mut self, session_id: &str, tool_name: &str) {
        let stats = self.get_or_create(session_id);
        stats.total_tool_calls += 1;
        *stats.tools_used.entry(tool_name.to_string()).or_insert(0) += 1;
        self.aggregate.total_tool_calls += 1;
    }

    fn get_or_create(&mut self, session_id: &str) -> &mut SessionStats {
        if !self.session_stats.contains_key(session_id) {
            self.aggregate.total_sessions += 1;
        }
        self.session_stats.entry(session_id.to_string()).or_insert_with(|| {
            SessionStats {
                total_turns: 0, total_tool_calls: 0, total_errors: 0,
                session_start: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
                tools_used: HashMap::new(),
            }
        })
    }

    pub fn get_aggregate(&self) -> &AggregateStats {
        &self.aggregate
    }
}
