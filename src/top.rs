use std::collections::HashMap;
use std::fs;
use std::path::Path;

use comfy_table::{Cell, ContentArrangement, Table};

pub struct AppMemEntry {
    pub name: String,
    pub rss_kb: u64,
}

/// Read PSS (Proportional Set Size) from smaps_rollup.
/// PSS is more accurate than RSS because it accounts for shared memory
/// proportionally — a shared 1MB region across 10 processes counts as 100KB
/// per process. This avoids the double-counting problem where RSS sums the
/// full shared region for every PID that maps it (e.g. Firefox with 26 PIDs).
fn read_pss_kb(pid: u32) -> Option<u64> {
    let path = format!("/proc/{}/smaps_rollup", pid);
    let data = fs::read_to_string(path).ok()?;
    for line in data.lines() {
        if line.starts_with("Pss:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return parts[1].parse().ok();
            }
        }
    }
    None
}

fn app_name(pid: u32) -> Option<String> {
    let exe_path = format!("/proc/{}/exe", pid);
    if let Ok(link) = fs::read_link(&exe_path) {
        if let Some(name) = link.file_name() {
            let name_str = name.to_string_lossy();
            // Strip common " (deleted)" suffix
            let name_str = name_str.trim_end_matches(" (deleted)");
            if !name_str.is_empty() {
                return Some(name_str.to_string());
            }
        }
    }
    // Fallback: cmdline[0] basename
    let cmdline_path = format!("/proc/{}/cmdline", pid);
    let data = fs::read(&cmdline_path).ok()?;
    if data.is_empty() {
        return None;
    }
    let arg0 = data.split(|&b| b == 0).next()?;
    if arg0.is_empty() {
        return None;
    }
    let s = String::from_utf8_lossy(arg0);
    Path::new(s.as_ref())
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
}

pub fn gather_top(n: usize) -> Vec<AppMemEntry> {
    let mut totals: HashMap<String, u64> = HashMap::new();

    let proc_dir = match fs::read_dir("/proc") {
        Ok(d) => d,
        Err(_) => return vec![],
    };

    for entry in proc_dir.flatten() {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        let pid: u32 = match name.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let rss = match read_pss_kb(pid) {
            Some(r) => r,
            None => continue,
        };

        let app = match app_name(pid) {
            Some(a) => a,
            None => {
                if rss == 0 {
                    continue;
                }
                format!("PID:{}", pid)
            }
        };

        *totals.entry(app).or_insert(0) += rss;
    }

    let mut entries: Vec<AppMemEntry> = totals
        .into_iter()
        .map(|(name, rss_kb)| AppMemEntry { name, rss_kb })
        .collect();

    entries.sort_by(|a, b| b.rss_kb.cmp(&a.rss_kb));
    entries.truncate(n);
    entries
}

fn format_size(kb: u64) -> String {
    if kb >= 1024 * 1024 {
        format!("{:.1} GiB", kb as f64 / (1024.0 * 1024.0))
    } else if kb >= 1024 {
        format!("{:.0} MiB", kb as f64 / 1024.0)
    } else {
        format!("{} KiB", kb)
    }
}

fn bar(fraction: f64, width: usize) -> String {
    let filled = ((fraction * width as f64).round() as usize).min(width);
    "█".repeat(filled)
}

pub fn format_top(entries: &[AppMemEntry]) -> String {
    crossterm::style::force_color_output(true);

    if entries.is_empty() {
        return "No processes found.".to_string();
    }

    let max_rss = entries[0].rss_kb;
    const BAR_WIDTH: usize = 8;

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Disabled);
    table.set_width(60);
    table.enforce_styling();
    table.set_header(vec!["APP", "RAM", "%"]);

    for entry in entries {
        let fraction = if max_rss > 0 {
            entry.rss_kb as f64 / max_rss as f64
        } else {
            0.0
        };
        let b = bar(fraction, BAR_WIDTH);
        table.add_row(vec![
            Cell::new(&entry.name),
            Cell::new(format_size(entry.rss_kb)),
            Cell::new(b),
        ]);
    }

    table.to_string()
}
