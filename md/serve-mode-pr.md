### PR Title

feat(mcp): serve mode over SSE via rust-mcp-sdk (tools.list/call, resources.list/read) + E2E smoke test

### PR Description (copy/paste this body)

- **Summary**: Introduces MCP “serve mode” for `golem-cli` using `rust-mcp-sdk` over SSE transport. Exposes CLI subcommands as tools and discoverable `golem.yaml` files as resources. Streamable HTTP is deferred.

- **Scope**:
  - **Feature flag**: `mcp-serve`
  - **Transport**: SSE at `/sse`
  - **Tools**: `tools.list` and `tools.call` wired to CLI subcommands
  - **Resources**: `resources.list` and `resources.read` wired to file:// discovery for `golem.yaml`
  - **Tests**: E2E SSE smoke test
  - **Docs**: Usage, tests, and progress checklist

- **Why a new PR**: Replaces the earlier scaffold PR with full SSE wiring and tests; the previous one was closed pending full implementation. See the closed scaffold PR [#319](https://github.com/golemcloud/golem-cli/pull/319).

- **Usage (local)**:

```bash
cargo build -p golem-cli --features mcp-serve
RUST_LOG=info cargo run -p golem-cli --features mcp-serve -- --serve --serve-port 1232
# SSE endpoint
# http://127.0.0.1:1232/sse
```

- **Tests**:
  - E2E SSE smoke:

```bash
cargo test -p golem-cli --features mcp-serve --test mcp_serve_smoke
```

  - Streamable HTTP roundtrip test exists, but is `#[ignore]` by design (SSE is the required transport for this PR).

- **Key files**:
  - `golem-cli/Cargo.toml` (feature, deps pin)
  - `golem-cli/src/serve/mod.rs` (MCP server + handlers)
  - `golem-cli/src/command.rs`; `golem-cli/src/command_handler/mod.rs` (serve mode CLI handling)
  - `golem-cli/tests/mcp_serve_smoke.rs` (SSE test)
  - `golem-cli/tests/mcp_stream_http.rs` (`#[ignore]`)
  - `md/serve-mode-overview.md`, `md/e2e-tests-mcp.md`, `md/implementation-checklist.md`, `md/serve-mode-progress.md`

- **Notes**:
  - SDK: `rust-mcp-sdk = "=0.5.1"` with features `["server", "macros", "hyper-server", "2025_06_18"]`
  - Transport: SSE only; HTTP deferred
  - Please enable “Allow edits by maintainers”

- **Checklist**:
  - [x] Feature flag and deps
  - [x] SSE server starts and responds
  - [x] tools.list/call
  - [x] resources.list/read
  - [x] E2E SSE smoke test
  - [x] Docs updated

- **Related/Links**:
  - Previously closed scaffold PR: [#319](https://github.com/golemcloud/golem-cli/pull/319)
  - Open new PR from this branch: `https://github.com/fjkiani/golem-cli/pull/new/feat/mcp-serve-sse-clean`

### Branches
- Base: `golemcloud/golem-cli:main`
- Head: `fjkiani:golem-cli:feat/mcp-serve-sse-clean` 