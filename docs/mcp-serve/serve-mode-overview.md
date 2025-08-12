# Serve Mode (MCP) – Overview and Decisions

- Start MCP server when `--serve` is passed; listen at `--serve-port`.
- Transport: SSE over HTTP on the provided port (compatible with popular agents and simple to test).
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