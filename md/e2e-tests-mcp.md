# E2E Tests – MCP Serve Mode

## Prerequisites
- Build with `mcp-serve` feature

```
cargo build -p golem-cli --features mcp-serve
```

## SSE smoke test
- Verifies the server starts and the `/sse` endpoint responds

Run:
```
cargo test -p golem-cli --features mcp-serve --test mcp_serve_smoke -- --nocapture
```

Expected:
- 1 test passes

Notes:
- Streamable HTTP tests are deferred; SSE is the supported transport now. 