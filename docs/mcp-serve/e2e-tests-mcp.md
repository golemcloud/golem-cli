# End-to-End Tests (MCP)

Server boot
- Launch `golem-cli --serve --serve-port <free_port>` in a temp dir.
- Wait for stdout line: `golem-cli running MCP Server at port`.

Client
- Use rust-mcp-sdk client (SSE) to connect.
- Verify tools:
  - `tools.list` contains expected leaves (e.g., `completion`, `profile.*`, or other safe commands).
  - `tools.call` with `args` executes and returns success; choose a command with deterministic output.
- Verify resources:
  - Create `golem.yaml` in cwd; ensure it appears in `resources.list`.
  - Add additional manifests in an ancestor and in a child dir; ensure both appear and `resources.read` returns correct content.

Reliability
- Avoid external network dependencies.
- Use timeouts and graceful shutdown.
- Run in CI with predictable, isolated temp dirs and ports.

Reference client SDK: `https://github.com/rust-mcp-stack/rust-mcp-sdk` 