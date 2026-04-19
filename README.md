# reap

Find what's running. Kill it. Move on.

`reap` is a human-centric process and port inspector that replaces `lsof` and `fuser` with human-readable output.

## Quick Start

```bash
reap port 8080          # Find process on port 8080
reap port 53 --udp      # Find process on UDP port 53
reap kill 8080           # Kill process on port 8080
reap kill -9 8080       # Force kill (SIGKILL)
reap listen             # Show all listening ports
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
| `reap kill <PORT>` | Kill process on port (SIGTERM) |
| `reap kill -9 <PORT>` | Force kill process (SIGKILL) |
| `reap listen` | Show all listening ports |
| `reap file <PATH>` | Find processes with file open |
| `reap name <NAME>` | Find processes by name |

## Output Format

| Column | Description |
|--------|-------------|
| PID | Process ID |
| USER | Username |
| COMMAND | Full command path |
| PORT | Port number |
| PROTOCOL | tcp or udp |

Use `-v` flag for verbose output with UID, inode, and file descriptor.
