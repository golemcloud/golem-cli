# CLI Flags and Entry – Serve Mode

Flags (in `GolemCliGlobalFlags` within `golem-cli/src/command.rs`):
- `--serve` (bool)
- `--serve-port <u16>` (default: 1232)

Entry flow (in `golem-cli/src/command_handler/mod.rs`):
- After parsing `GolemCliCommand`, if `global_flags.serve` is true:
  - Construct context as usual.
  - Call `serve::run_server(ctx, port).await`.
  - Print: `golem-cli running MCP Server at port {port}`.
  - Return `ExitCode::SUCCESS`.

Server module
- Add `golem-cli/src/serve/mod.rs`:
  - Build an `rust-mcp-sdk` server (SSE over HTTP/Hyper) on `serve_port`.
  - Register tools (from Clap).
  - Register resources (manifest files).
  - Run until cancelled (SIGINT), graceful shutdown. 