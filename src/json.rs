use crate::port::PortEntry;
use serde::Serialize;

/// JSON output for a port query result
#[derive(Serialize)]
pub struct PortResult {
    pub port: u16,
    pub protocol: String,
    pub processes: Vec<ProcessInfo>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct ProcessInfo {
    pub pid: i32,
    pub user: String,
    pub command: String,
    pub state: String,
}

impl PortResult {
    pub fn from_entries(entries: &[PortEntry], port: u16, protocol: &str) -> Self {
        let processes: Vec<ProcessInfo> = entries.iter().map(|e| {
            let user = match uzers::get_user_by_uid(e.uid) {
                Some(u) => u.name().to_string_lossy().to_string(),
                None => e.uid.to_string(),
            };
            ProcessInfo {
                pid: e.pid,
                user,
                command: e.command.clone(),
                state: e.state.to_string(),
            }
        }).collect();
        
        PortResult {
            port,
            protocol: protocol.to_string(),
            count: processes.len(),
            processes,
        }
    }
}

pub fn format_json(entries: &[PortEntry], port: u16, protocol: &str) -> String {
    let result = PortResult::from_entries(entries, port, protocol);
    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{\"error\": \"failed to serialize\"}".to_string())
}

pub fn format_top_json(entries: &[crate::top::AppMemEntry]) -> String {
    serde_json::to_string_pretty(entries).unwrap_or_else(|_| "[{\"error\": \"failed to serialize\"}]".to_string())
}
