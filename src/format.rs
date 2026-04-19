use colored::Colorize;
use comfy_table::{Cell, Color, ContentArrangement, Table};

use crate::port::PortEntry;

pub fn format_port_entries(entries: &[PortEntry], verbose: bool) -> String {
    crossterm::style::force_color_output(true);
    if entries.is_empty() {
        return String::new();
    }

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_width(80);
    table.enforce_styling();

    let mut header = vec!["PID".to_string(), "USER".to_string(), "COMMAND".to_string()];
    if verbose {
        header.push("UID".to_string());
        header.push("INODE".to_string());
        header.push("FD".to_string());
    }
    header.push("PORT".to_string());
    header.push("PROTOCOL".to_string());

    table.set_header(header);

    let max_command_len = if verbose { 40 } else { 50 };

    for entry in entries {
        let user = match uzers::get_user_by_uid(entry.uid) {
            Some(u) => u.name().to_string_lossy().to_string(),
            None => entry.uid.to_string(),
        };

        let command = truncate_command(&entry.command, max_command_len);

        let color = match entry.state.as_str() {
            "LISTEN" => Color::Green,
            "ESTABLISHED" => Color::Yellow,
            _ => Color::Red,
        };

        let mut row: Vec<Cell> = Vec::with_capacity(if verbose { 9 } else { 5 });

        row.push(Cell::new(entry.pid.to_string()));
        row.push(Cell::new(user));
        row.push(Cell::new(command));

        if verbose {
            row.push(Cell::new(entry.uid.to_string()));
            row.push(Cell::new(entry.inode.to_string()));
            row.push(Cell::new(&entry.fd));
        }

        row.push(Cell::new(entry.local_port.to_string()).fg(color));
        row.push(Cell::new(&entry.protocol).fg(color));

        table.add_row(row);
    }

    table.to_string()
}

fn truncate_command(cmd: &str, max_len: usize) -> String {
    if cmd.len() <= max_len {
        cmd.to_string()
    } else {
        format!("{}...", &cmd[..max_len.saturating_sub(3)])
    }
}

pub fn print_error(msg: &str) {
    eprintln!("{}", msg.red());
}

pub fn print_success(msg: &str) {
    println!("{}", msg.green());
}

pub fn is_terminal() -> bool {
    atty::is(atty::Stream::Stdout)
}

pub fn strip_ansi(s: &str) -> String {
    s.replace('\x1b', "")
}
