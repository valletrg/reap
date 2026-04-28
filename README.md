# reap

Find what's running. Kill it. Move on.

`reap` is a human-centric process and port inspector that replaces `lsof` and `fuser` with human-readable output.

## Quick Start

```bash
reap port 8080          # Find process on port 8080
reap port 53 --udp      # Find process on UDP port 53
reap port 8080 --json   # JSON output for scripting
reap kill 8080           # Kill process on port 8080
reap kill -9 8080       # Force kill (SIGKILL)
reap listen             # Show all listening ports
reap which 8080         # Quick one-liner: "Port 8080 is owned by node (PID 12345)"
reap watch 8080         # Live monitoring, refresh every 1s
reap watch 8080 --interval 2 --udp  # Watch UDP, refresh every 2s
reap top                # Top RAM usage by app
reap top --json         # JSON output for top
```

## The Problem

Port 3000 is already in use. Again. Stop Googling the `lsof` flags.

Old way:
```bash
$ lsof -iTCP:8080 -sTCP:LISTEN
node  24523 alice  23u  IPv4  570123  0t0  TCP *:http-alt (LISTEN)
```

New way:
```bash
$ reap port 8080
+-------+-------+----------------------------+------+----------+
| PID   | USER  | COMMAND                    | PORT | PROTOCOL |
+===========================================================+
| 24523 | alice | node /home/alice/app/server.js | 8080 | tcp      |
+-------+-------+----------------------------+------+----------+
```

## Features

- Intuitive syntax: `reap port`, `reap kill`
- Human-readable output (usernames instead of UIDs, clean paths)
- Color-coded state (green = LISTEN, yellow = ESTABLISHED, red = otherwise)
- Graceful without sudo (shows `<restricted>` for inaccessible processes)
- Written in Rust

## Installation
```bash
git clone https://github.com/yourusername/reap.git
cd reap
cargo build --release
cargo install --path . 

or

./target/release/reap
```

## Commands

| Command | Description |
|--------|-------------|
| `reap port <PORT>` | Find process using a port |
| `reap port <PORT> --udp` | Find process using a UDP port |
| `reap port <PORT> --json` | Output as JSON (for scripting) |
| `reap kill <PORT>` | Kill process on port (SIGTERM) |
| `reap kill -9 <PORT>` | Force kill process (SIGKILL) |
| `reap listen` | Show all listening ports |
| `reap which <PORT>` | Quick one-liner summary |
| `reap watch <PORT>` | Live monitoring (1s refresh) |
| `reap watch <PORT> --interval N` | Watch with N-second refresh |
| `reap watch <PORT> --udp` | Watch a UDP port |
| `reap file <PATH>` | Find processes with file open |
| `reap name <NAME>` | Find processes by name |
| `reap top` | Top RAM usage by app |
| `reap top --json` | Top RAM as JSON |
| `reap top -n <N>` | Show top N apps (default: 10) |

## Output Format

### Table Output (default)
| Column | Description |
|--------|-------------|
| PID | Process ID |
| USER | Username |
| COMMAND | Full command path |
| PORT | Port number |
| PROTOCOL | tcp or udp |

Use `-v` flag for verbose output with UID, inode, and file descriptor.

### JSON Output
Use `--json` flag with `port` or `top` commands for machine-readable output:
```bash
reap port 8080 --json
reap top --json
```

### Which Output
`reap which` outputs a simple one-liner:
```
Port 8080 is owned by node (PID 12345)
Port 8080 is not in use
Port 3000 is used by 3 processes (PIDs: [123, 456, 789])
```
