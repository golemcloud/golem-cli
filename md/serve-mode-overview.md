# Serve Mode (MCP) – Overview and Decisions

- Start MCP server when `--serve` is passed; listen at `--serve-port`.
- Transport: SSE over HTTP on the provided port (compatible with popular agents and simple to test).
  - SSE endpoint: `http://127.0.0.1:<port>/sse`
- Tool naming: `app.build`, `worker.invoke`, `api.get`, `component.deploy`, etc.
- Tool schema: generic `args: string[]` (+ optional `cwd?: string`, `stdin?: string`) to avoid hard-coding and keep the mapping DRY.
- Resource exposure: surface `golem.yaml` discovered in current dir, ancestors, and one-level children.
- DRY execution: generate tools from the Clap command tree; delegate execution to existing handlers.

Relevant code paths to integrate:
- `golem-cli/src/command.rs`
- `golem-cli/src/command_handler/mod.rs`
- `golem-cli/src/main.rs`
- `golem-cli/src/context.rs`

SDK
- Use `rust-mcp-sdk` (server + Hyper SSE). See README for features/handlers: `https://github.com/rust-mcp-stack/rust-mcp-sdk`

## Usage (local)

Build with feature:

```
cargo build -p golem-cli --features mcp-serve
```

Run serve mode (note the `--` before CLI flags):

```
cargo run -p golem-cli --features mcp-serve -- --serve --serve-port 1232
```

SSE endpoint:

```
http://127.0.0.1:1232/sse
```

Expected line on start:

```
golem-cli running MCP Server at port 1232
```

Notes:
- Optional streamable HTTP testing is deferred; SSE is the supported transport right now. 