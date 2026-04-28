# Repository Guidelines

This document provides guidelines for AI agents and contributors working on `reap`.

## Project Overview

`reap` is a human-centric process and port inspector CLI written in Rust. It replaces `lsof` and `fuser` with human-readable output. The project is located at `/mnt/storage/projects/reap`.

## Project Structure

```
reap/
├── src/
│   ├── main.rs        # CLI definition, argument parsing, command routing
│   ├── port.rs        # Port inspection via /proc/net/tcp parsing
│   ├── kill.rs        # Signal sending (SIGTERM/SIGKILL)
│   ├── format.rs      # Table formatting with color-coded output
│   ├── proc.rs        # Procfs helpers (cmdline, UID resolution)
│   ├── top.rs         # Top RAM usage by process
│   └── format_test.rs # Manual table formatting test
├── docs/
│   ├── index.md       # Project documentation
│   ├── guides/        # User guides
│   ├── reference/     # Reference docs
│   └── concepts/      # Conceptual documentation
├── Cargo.toml         # Rust dependencies
├── Makefile           # Development shortcuts
└── README.md          # User-facing documentation
```

### Key Entry Points

| File | Function | Description |
|------|----------|-------------|
| `src/main.rs` | `main()` (line 58) | CLI entry point |
| `src/port.rs` | `find_processes_by_port()` (line 76) | Port lookup logic |
| `src/kill.rs` | `kill_by_port()` (line 7) | Kill action |

## Building and Testing

```bash
cargo build --release           # Build optimized binary
./target/release/reap --help    # Verify build
cargo test                      # Run unit tests (if any)
```

**Note**: `format_test.rs` is a standalone test binary, not a unit test. Run it with:
```bash
cargo run --example format_test
```

## Coding Style

- **Language**: Rust (2021 edition)
- **Dependencies**: clap (CLI), comfy-table (tables), colored (colors), procfs (procfs bindings), nix (signals/process)
- **Indentation**: Standard Rust formatting via `cargo fmt`
- **Concurrency**: Async not used; synchronous I/O with procfs

## Testing Guidelines

- Unit tests use standard `#[test]` attributes
- `format_test.rs` is a manual test binary for table rendering
- Test naming: standard Rust conventions
- Run tests: `cargo test`

## Commit Message Conventions

Recent commits follow a simple pattern:
```
<verb> <affected component>: <brief description>
```

Examples:
- `added reap top`
- `add MIT license`
- `initial commit`

Keep messages concise and in lowercase.

## Pull Request Guidelines

1. **Title**: Clear, concise description of the change
2. **Description**: Explain what and why, not how
3. **Linked issues**: Reference any related issues
4. **Screenshots**: Include for UI/table output changes
5. **Testing**: Verify with `cargo build --release` before submitting

## Important Behaviors

- **Protocol**: All port queries use TCP by default. Use `--udp` for UDP.
- **Graceful degradation**: Inaccessible `/proc` info shows `<restricted>` but still displays PID/port.
- **Pipe detection**: When stdout is not a TTY, ANSI codes are stripped automatically.
- **Unimplemented commands**: `reap file` and `reap name` exist in CLI but return "not yet implemented".
- **Performance**: `reap listen` is slow (walks ports 1-65535); future optimization would parse `/proc/net/tcp` once.
