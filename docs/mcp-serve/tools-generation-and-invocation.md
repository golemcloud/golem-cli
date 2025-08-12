# Tools: Generation from Clap and Invocation

Discovery
- Use `GolemCliCommand::command()` to traverse the Clap command tree.
- For each leaf subcommand path, create a tool named by joining with dots (e.g., `app.build`).
- Tool description from `Command::get_about()` when available.

Schema
- Parameters:
  - `args: string[]`
  - Optional: `cwd?: string`, `stdin?: string`
- Keep generic to avoid per-command duplication.

Invocation
- Reconstruct argv: `[bin_name] + path_segments + args`.
- Execute by delegating to `CommandHandler::handle_args(argv, hooks)`.
- Capture stdout/stderr and exit status.
- If users include `--format json` in `args`, pass through to return structured JSON when supported by the command.

Errors
- Non-zero CLI exit maps to an MCP error with message and captured stderr. 