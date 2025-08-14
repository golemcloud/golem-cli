# Implementation Checklist (MCP Serve Mode)

See also: `md/serve-mode-progress.md` for per-item details (code paths, tests, notes).

- Flags and entry
  - [x] Add `--serve` and `--serve-port` to `GolemCliGlobalFlags`
  - [x] Early branch in handler to start serve mode and print startup line
  - [x] Create `golem-cli/src/serve/mod.rs` stub

- Server scaffolding
  - [x] Enumerate tools from Clap
  - [x] Discover manifest resources (cwd/ancestors/children)
  - [x] Placeholder server that waits for Ctrl-C
  - [x] Integrate `rust-mcp-sdk` SSE server with handlers (behind `mcp-serve`)

- Tool execution
  - [x] Build argv for tool paths
  - [x] Programmatic invocation via `CommandHandler`
  - [x] Subprocess execution helper (capture stdout/stderr)
  - [x] Wire MCP `tools.call` → programmatic or subprocess execution

- Resources
  - [x] List discovery (golem.yaml)
  - [x] MCP resources.list implementation
  - [x] MCP resources.read implementation

- Tests
  - [ ] E2E tests for tools.list and tools.call
  - [ ] E2E tests for resources.list/read
  - [ ] Ensure no external dependencies and add timeouts

- Dependencies
  - [x] Add `rust-mcp-sdk` server + SSE features to `golem-cli/Cargo.toml`

Reference: `https://github.com/rust-mcp-stack/rust-mcp-sdk` 