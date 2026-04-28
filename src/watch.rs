use std::time::Duration;
use std::io::{self, Write};

use crossterm::{ExecutableCommand, cursor, terminal::{self, ClearType}};
use crossterm::event::{poll, read, Event, KeyCode, KeyModifiers};

use crate::port::find_processes_by_port;
use crate::format::format_port_entries;

pub fn watch_port(port: u16, protocol: &str, interval_secs: u64) {
    let mut first_run = true;
    
    loop {
        if first_run {
            first_run = false;
        } else {
            // wait for interval or until ctrl+c
            if poll(Duration::from_secs(interval_secs)).unwrap() {
                if let Ok(Event::Key(key)) = read() {
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        // Restore terminal and exit
                        let _ = io::stdout().execute(terminal::Clear(ClearType::All));
                        let _ = io::stdout().execute(cursor::Show);
                        println!("\nWatch stopped.");
                        return;
                    }
                }
            }
        }
        
        let _ = io::stdout().execute(terminal::Clear(ClearType::All));
        let _ = io::stdout().execute(cursor::MoveTo(0, 0));
        let _ = io::stdout().execute(cursor::Hide);
        
        let proto_display = if protocol == "udp" { "UDP" } else { "TCP" };
        println!("Watching port {} ({}) - Press Ctrl+C to stop\n", port, proto_display);
        
        // find and display processes
        let entries = find_processes_by_port(port, protocol);
        let output = format_port_entries(&entries, false);
        
        if output.is_empty() {
            println!("No process found on port {}. Waiting...", port);
        } else {
            println!("{}", output);
        }
        
        let _ = io::stdout().flush();
    }
}

pub fn which_port(port: u16, protocol: &str) -> String {
    let entries = find_processes_by_port(port, protocol);
    
    if entries.is_empty() {
        format!("Port {} is not in use", port)
    } else if entries.len() == 1 {
        let e = &entries[0];
        let exe = e.command.split(' ').next().unwrap_or("?");
        format!("Port {} is owned by {} (PID {})", port, exe, e.pid)
    } else {
        let pids: Vec<i32> = entries.iter().map(|e| e.pid).collect();
        format!("Port {} is used by {} processes (PIDs: {:?})", port, entries.len(), pids)
    }
}
