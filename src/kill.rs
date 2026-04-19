use colored::Colorize;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

use crate::port::find_processes_by_port;

pub fn kill_by_port(port: u16, signal_num: i32) {
    let entries = find_processes_by_port(port, "tcp");

    if entries.is_empty() {
        eprintln!("{} No process found on port {}", "ERROR".red(), port);
        return;
    }

    let signal = match signal_num {
        9 => Signal::SIGKILL,
        _ => Signal::SIGTERM,
    };

    let action = if signal == Signal::SIGKILL {
        "Killed"
    } else {
        "Terminated"
    };

    for entry in entries {
        let pid = Pid::from_raw(entry.pid);

        match kill(pid, signal) {
            Ok(()) => {
                println!(
                    "{} {} ({}) on port {}/tcp",
                    action.green(),
                    entry.pid,
                    entry.command.split(' ').next().unwrap_or("?"),
                    port
                );
            }
            Err(e) => {
                eprintln!(
                    "{} Failed to {} {} on port {}/tcp: {}",
                    "ERROR".red(),
                    action.to_lowercase(),
                    entry.pid,
                    port,
                    e
                );
            }
        }
    }
}
