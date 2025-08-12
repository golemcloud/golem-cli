# Logging and UX for Serve Mode

- On serve start, print to stdout:
  - `golem-cli running MCP Server at port {port}`
- Keep logs concise in serve mode; use info-level for lifecycle, debug for details.
- Do not alter default formatting for other CLI commands.
- Ensure clean shutdown messaging on SIGINT. 