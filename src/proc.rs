use std::fs;
use std::path::Path;

pub fn resolve_fd_symlink(pid: i32, fd: &str) -> Option<String> {
    let path_str = format!("/proc/{}/fd/{}", pid, fd);
    let path = Path::new(&path_str);
    fs::read_link(path).ok().and_then(|p| p.to_str().map(String::from))
}

pub fn read_cmdline(pid: i32) -> Option<String> {
    let path_str = format!("/proc/{}/cmdline", pid);
    let path = Path::new(&path_str);
    let data = fs::read(path).ok()?;
    if data.is_empty() {
        return None;
    }
    let cmdline = data
        .split(|&b| b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).to_string())
        .collect::<Vec<_>>()
        .join(" ");
    if cmdline.is_empty() {
        None
    } else {
        Some(cmdline)
    }
}

pub fn get_uid(pid: i32) -> Option<u32> {
    let path_str = format!("/proc/{}/status", pid);
    let path = Path::new(&path_str);
    let data = fs::read_to_string(path).ok()?;
    for line in data.lines() {
        if line.starts_with("Uid:") {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if parts.len() >= 2 {
                return parts[1].parse().ok();
            }
        }
    }
    None
}

pub fn inode_for_fd(pid: i32, fd: &str) -> Option<u64> {
    let link_target = resolve_fd_symlink(pid, fd)?;

    if let Some(inode) = parse_socket_inode(&link_target) {
        return Some(inode);
    }

    let path_str = format!("/proc/{}/fd/{}", pid, fd);
    let path = Path::new(&path_str);
    use std::os::unix::fs::MetadataExt;
    fs::metadata(path).ok().map(|m| m.ino())
}

fn parse_socket_inode(target: &str) -> Option<u64> {
    let target = target.trim();
    if target.starts_with("socket:[") && target.ends_with(']') {
        let inner = &target[8..target.len() - 1];
        inner.parse().ok()
    } else {
        None
    }
}
