# MCP Serve Mode – Progress Log

This log records each checklist item we complete, how it was achieved, impacted code paths, and test coverage.

| Item | Status | Code paths | Tests | Notes | Commit/PR |
|------|--------|-----------|-------|-------|-----------|
| SSE smoke test (server starts, /sse reachable) | done | `golem-cli/src/serve/mod.rs` (HyperServerOptions `custom_sse_endpoint`) | `golem-cli/tests/mcp_serve_smoke.rs` | Uses Accept: `text/event-stream`; timeout 5s; passes locally (feature `mcp-serve`) | local branch |
| Integrate `rust-mcp-sdk` SSE server with handlers (behind `mcp-serve`) | done | `golem-cli/src/serve/mod.rs` (mcp_server), `golem-cli/Cargo.toml` (features) | TODO: E2E client smoke | Compiles and runs; imports updated to 0.5 API | local branch |
| MCP resources.list implementation | done | `golem-cli/src/serve/mod.rs` (`handle_list_resources_request`) | TODO: E2E list | Uses file:// URIs for discovered `golem.yaml` | local branch |
| MCP resources.read implementation | done | `golem-cli/src/serve/mod.rs` (`handle_read_resource_request`) | TODO: E2E read | Returns `TextResourceContents` with `mimeType: application/yaml` | local branch |
| tools.list wiring | done | `golem-cli/src/serve/mod.rs` (`handle_list_tools_request`) | TODO | Uses `discover_tools_from_clap()` | - |
| tools.call wiring | done | `golem-cli/src/serve/mod.rs` (`handle_call_tool_request`) | TODO | Uses `run_cli_subprocess()` and returns `structured_content` (status/stdout/stderr) | - |
| E2E tests (tools/resources) | TODO | `golem-cli/tests/` (new) | TODO | Bound timeouts; no external deps | - |
| Docs usage update | done | `md/serve-mode-overview.md` (Usage section) | n/a | Added build/run commands with `--` | local branch |

Conventions:
- Code paths are relative to repo root.
- Tests should reference exact files and test names when added.
- Link commits/PRs when pushed upstream. 