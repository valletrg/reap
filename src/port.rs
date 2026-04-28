use std::collections::HashMap;
use std::fs;

use crate::proc::{get_uid, inode_for_fd, read_cmdline, resolve_fd_symlink};

#[derive(Debug, Clone)]
pub struct PortEntry {
    pub pid: i32,
    pub uid: u32,
    pub command: String,
    #[allow(dead_code)]
    pub local_addr: String,
    pub local_port: u16,
    pub protocol: String,
    pub state: String,
    pub inode: u64,
    pub fd: String,
    #[allow(dead_code)]
    pub mode: Option<String>,
}

pub fn parse_proc_net_tcp(content: &str) -> Vec<(String, u16, u64, String)> {
    let mut entries = Vec::new();
    for line in content.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }
        let local_addr_hex = parts[1];
        let local_port_hex = parts[1];
        let inode_str = parts[9];
        let state_hex = parts[3];

        let local_port = u16::from_str_radix(&local_port_hex[local_port_hex.len() - 4..], 16).ok();

        let inode: Option<u64> = inode_str.parse().ok();

        let state = u8::from_str_radix(state_hex, 16).ok();

        let local_addr = hex_to_ip(&local_addr_hex[..local_addr_hex.len() - 5]);

        if let (Some(port), Some(ino), Some(st)) = (local_port, inode, state) {
            entries.push((local_addr, port, ino, tcp_state_string(st)));
        }
    }
    entries
}

pub fn hex_to_ip(hex: &str) -> String {
    let parts: Vec<u32> = hex
        .split(':')
        .filter_map(|s| u32::from_str_radix(s, 16).ok())
        .collect();
    if parts.len() == 4 {
        format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], parts[3])
    } else {
        String::from("unknown")
    }
}

pub fn tcp_state_string(state: u8) -> String {
    match state {
        0x01 => "ESTABLISHED",
        0x02 => "SYN_SENT",
        0x03 => "SYN_RECV",
        0x04 => "FIN_WAIT1",
        0x05 => "FIN_WAIT2",
        0x06 => "TIME_WAIT",
        0x07 => "CLOSE",
        0x08 => "CLOSE_WAIT",
        0x09 => "LAST_ACK",
        0x0A => "LISTEN",
        0x0B => "CLOSING",
        _ => "UNKNOWN",
    }
    .to_string()
}

fn build_inode_map() -> HashMap<u64, (i32, String, u32, String)> {
    let mut inode_to_info: HashMap<u64, (i32, String, u32, String)> = HashMap::new();

    if let Ok(dir) = fs::read_dir("/proc") {
        for entry in dir.flatten() {
            let name = entry.file_name();
            let pid_str = name.to_string_lossy();
            if let Ok(pid) = pid_str.parse::<i32>() {
                if let Ok(fd_dir) = fs::read_dir(format!("/proc/{}/fd", pid)) {
                    for fd_entry in fd_dir.flatten() {
                        let fd_name = fd_entry.file_name().to_string_lossy().to_string();
                        if let Some(inode) = inode_for_fd(pid, &fd_name) {
                            let cmdline = read_cmdline(pid).unwrap_or_else(|| "<restricted>".to_string());
                            let uid = get_uid(pid).unwrap_or(0);
                            inode_to_info.entry(inode).or_insert((pid, cmdline, uid, fd_name));
                        }
                    }
                }
            }
        }
    }

    inode_to_info
}

pub fn find_processes_by_port(port: u16, protocol: &str) -> Vec<PortEntry> {
    let entries: Vec<(String, u16, u64, String)> = if protocol == "udp" {
        let udp_content = fs::read_to_string("/proc/net/udp").unwrap_or_default();
        let udp6_content = fs::read_to_string("/proc/net/udp6").unwrap_or_default();
        let udp_entries = parse_proc_net_tcp(&udp_content);
        let udp6_entries = parse_proc_net_tcp(&udp6_content);
        udp_entries.into_iter().chain(udp6_entries).collect()
    } else {
        let tcp_content = fs::read_to_string("/proc/net/tcp").unwrap_or_default();
        let tcp6_content = fs::read_to_string("/proc/net/tcp6").unwrap_or_default();
        let tcp_entries = parse_proc_net_tcp(&tcp_content);
        let tcp6_entries = parse_proc_net_tcp(&tcp6_content);
        tcp_entries.into_iter().chain(tcp6_entries).collect()
    };

    let port_entries: Vec<(String, u16, u64, String)> = entries
        .into_iter()
        .filter(|(_, p, _, _)| *p == port)
        .collect();

    let inode_to_info = build_inode_map();

    let mut results = Vec::new();
    for (local_addr, local_port, inode, state) in port_entries {
        if let Some((pid, command, uid, fd)) = inode_to_info.get(&inode) {
            results.push(PortEntry {
                pid: *pid,
                uid: *uid,
                command: command.clone(),
                local_addr: local_addr.clone(),
                local_port,
                protocol: protocol.to_string(),
                state: state.clone(),
                inode,
                fd: fd.clone(),
                mode: None,
            });
        }
    }

    results
}

pub fn find_all_listening() -> Vec<PortEntry> {
    let tcp_content = fs::read_to_string("/proc/net/tcp").unwrap_or_default();
    let tcp6_content = fs::read_to_string("/proc/net/tcp6").unwrap_or_default();
    let tcp_entries = parse_proc_net_tcp(&tcp_content);
    let tcp6_entries = parse_proc_net_tcp(&tcp6_content);

    let all_entries: Vec<(String, u16, u64, String)> = tcp_entries
        .into_iter()
        .chain(tcp6_entries)
        .filter(|(_, _, _, state)| state == "LISTEN")
        .collect();

    let inode_to_info = build_inode_map();

    let mut results = Vec::new();
    for (local_addr, local_port, inode, state) in all_entries {
        if let Some((pid, command, uid, fd)) = inode_to_info.get(&inode) {
            results.push(PortEntry {
                pid: *pid,
                uid: *uid,
                command: command.clone(),
                local_addr: local_addr.clone(),
                local_port,
                protocol: "tcp".to_string(),
                state: state.clone(),
                inode,
                fd: fd.clone(),
                mode: None,
            });
        }
    }

    results
}

pub fn find_processes_by_file(path: &str) -> Vec<PortEntry> {
    let canonical_path = std::fs::canonicalize(path).ok().map(|p| p.to_string_lossy().to_string());
    let target = canonical_path.as_deref().unwrap_or(path);

    let mut results = Vec::new();

    if let Ok(dir) = fs::read_dir("/proc") {
        for entry in dir.flatten() {
            let name = entry.file_name();
            let pid_str = name.to_string_lossy();
            if let Ok(pid) = pid_str.parse::<i32>() {
                if let Ok(fd_dir) = fs::read_dir(format!("/proc/{}/fd", pid)) {
                    for fd_entry in fd_dir.flatten() {
                        let fd_name = fd_entry.file_name().to_string_lossy().to_string();
                        if let Some(resolved) = resolve_fd_symlink(pid, &fd_name) {
                            if resolved == target || resolved.starts_with(&format!("{}:", target)) {
                                let cmdline = read_cmdline(pid).unwrap_or_else(|| "<restricted>".to_string());
                                let uid = get_uid(pid).unwrap_or(0);
                                let mode = if fd_name.ends_with('r') {
                                    Some("READ".to_string())
                                } else if fd_name.ends_with('w') {
                                    Some("WRITE".to_string())
                                } else if fd_name.ends_with('u') {
                                    Some("READ/WRITE".to_string())
                                } else {
                                    None
                                };
                                results.push(PortEntry {
                                    pid,
                                    uid,
                                    command: cmdline,
                                    local_addr: "N/A".to_string(),
                                    local_port: 0,
                                    protocol: "file".to_string(),
                                    state: "OPEN".to_string(),
                                    inode: 0,
                                    fd: fd_name,
                                    mode,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    results
}

pub fn find_processes_by_name(name: &str) -> Vec<PortEntry> {
    let name_lower = name.to_lowercase();
    let mut results = Vec::new();

    if let Ok(dir) = fs::read_dir("/proc") {
        for entry in dir.flatten() {
            let name = entry.file_name();
            let pid_str = name.to_string_lossy();
            if let Ok(pid) = pid_str.parse::<i32>() {
                if let Some(cmdline) = read_cmdline(pid) {
                    let cmdline_lower = cmdline.to_lowercase();
                    if cmdline_lower.contains(&name_lower) {
                        let uid = get_uid(pid).unwrap_or(0);
                        results.push(PortEntry {
                            pid,
                            uid,
                            command: cmdline,
                            local_addr: "N/A".to_string(),
                            local_port: 0,
                            protocol: "any".to_string(),
                            state: "N/A".to_string(),
                            inode: 0,
                            fd: "N/A".to_string(),
                            mode: None,
                        });
                    }
                }
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_ip_localhost() {
        let result = hex_to_ip("7F000001");
        assert_eq!(result, "unknown"); // single segment, not 4
    }

    #[test]
    fn test_hex_to_ip_multiple_segments() {
        // "01020304" without colons splits to ["01020304"] only 1 element, returns "unknown"
        let result = hex_to_ip("01020304");
        assert_eq!(result, "unknown");
    }

    #[test]
    fn test_hex_to_ip_with_colons() {
        // four parts separated by colons, actual format from /proc/net/tcp
        let result = hex_to_ip("01:02:03:04");
        assert_eq!(result, "1.2.3.4");
    }

    #[test]
    fn test_hex_to_ip_invalid() {
        let result = hex_to_ip("invalid");
        assert_eq!(result, "unknown");
    }

    #[test]
    fn test_hex_to_ip_partial() {
        // Only 2 parts
        let result = hex_to_ip("0102");
        assert_eq!(result, "unknown");
    }

    #[test]
    fn test_hex_to_ip_three_parts() {
        // Only 3 parts
        let result = hex_to_ip("010203");
        assert_eq!(result, "unknown");
    }

    #[test]
    fn test_tcp_state_string_listen() {
        assert_eq!(tcp_state_string(0x0A), "LISTEN");
    }

    #[test]
    fn test_tcp_state_string_established() {
        assert_eq!(tcp_state_string(0x01), "ESTABLISHED");
    }

    #[test]
    fn test_tcp_state_string_unknown() {
        assert_eq!(tcp_state_string(0xFF), "UNKNOWN");
    }

    #[test]
    fn test_parse_proc_net_tcp_valid_line() {
        let content = "  sl  local_address rem_address  st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode\n  0: 00000000:0050 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000 0 62915 1 c200000000000000 100 0 0 10 -1";
        let entries = parse_proc_net_tcp(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].1, 80); // port 0x0050 = 80
        assert_eq!(entries[0].3, "LISTEN"); // state 0x0A = LISTEN
    }

    #[test]
    fn test_parse_proc_net_tcp_empty_content() {
        let entries = parse_proc_net_tcp("");
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_parse_proc_net_tcp_header_only() {
        let entries = parse_proc_net_tcp("sl  local_address rem_address  st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode");
        assert_eq!(entries.len(), 0);
    }
}
