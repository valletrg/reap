use clap::{Parser, Subcommand};

mod format;
mod kill;
mod port;
mod proc;

use colored::control::set_override;
use format::{format_port_entries, print_error};
use kill::kill_by_port;
use port::{find_all_listening, find_processes_by_file, find_processes_by_name, find_processes_by_port};

#[derive(clap::Parser)]
#[command(
    name = "reap",
    about = "Human-centric process and port inspector",
    version = "0.1.0"
)]
struct Cli {
    #[arg(long, value_enum, default_value = "auto")]
    color: ColorOption,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::ValueEnum, Clone, Default)]
enum ColorOption {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Find processes by port")]
    Port {
        port: u16,
        #[arg(long, short)]
        udp: bool,
        #[arg(long, short)]
        verbose: bool,
    },
    #[command(about = "Find processes by file")]
    File {
        path: String,
        #[arg(long, short)]
        verbose: bool,
    },
    #[command(about = "Find processes by name")]
    Name {
        name: String,
        #[arg(long, short)]
        verbose: bool,
    },
    #[command(about = "Find processes listening on a port")]
    Listen {
        #[arg(long, short)]
        verbose: bool,
    },
    #[command(about = "Kill process on port")]
    Kill {
        port: u16,
        /// Signal number to send (default: 15 for SIGTERM, use 9 for SIGKILL)
        #[arg(default_value = "15", allow_hyphen_values = true)]
        signal: i32,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.color {
        ColorOption::Always => set_override(true),
        ColorOption::Never => set_override(false),
        ColorOption::Auto => {}
    }

    match cli.command {
        Some(Commands::Port { port, udp, verbose }) => {
            let protocol = if udp { "udp" } else { "tcp" };
            let entries = find_processes_by_port(port, protocol);
            let output = format_port_entries(&entries, verbose);
            if output.is_empty() {
                print_error(&format!("No process found on port {}", port));
                std::process::exit(1);
            }
            println!("{}", output);
        }
        Some(Commands::Kill { signal, port }) => {
            kill_by_port(port, signal);
        }
        Some(Commands::Listen { verbose }) => {
            let entries = find_all_listening();
            let output = format_port_entries(&entries, verbose);
            if output.is_empty() {
                print_error("No listening ports found");
                std::process::exit(1);
            }
            println!("{}", output);
        }
        Some(Commands::File { path, verbose }) => {
            let entries = find_processes_by_file(&path);
            let output = format_port_entries(&entries, verbose);
            if output.is_empty() {
                print_error(&format!("No process found with file {}", path));
                std::process::exit(1);
            }
            println!("{}", output);
        }
        Some(Commands::Name { name, verbose }) => {
            let entries = find_processes_by_name(&name);
            let output = format_port_entries(&entries, verbose);
            if output.is_empty() {
                print_error(&format!("No process found with name {}", name));
                std::process::exit(1);
            }
            println!("{}", output);
        }
        None => {
            print_error("No command specified. Use reap --help for usage.");
            std::process::exit(1);
        }
    }
}
