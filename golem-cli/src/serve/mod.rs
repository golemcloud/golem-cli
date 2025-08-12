// Copyright 2024-2025 Golem Cloud
//
// Licensed under the Golem Source License v1.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://license.golem.cloud/LICENSE
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::context::Context;
use crate::log;
use crate::{command_name, command_handler::CommandHandler};
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{ExitCode, Stdio};
use std::sync::Arc;
use std::io;

#[cfg(feature = "mcp-serve")]
mod mcp_server {
    use super::*;
    use rust_mcp_sdk::server::prelude::*;
    use rust_mcp_sdk::server::server_runtime::create_server;

    pub async fn run(ctx: Arc<Context>, port: u16) -> anyhow::Result<()> {
        let tools = discover_tools_from_clap();
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let manifests = discover_manifest_resources(&cwd);

        // Build server with tools and resources
        let mut server = ServerBuilder::new("golem-cli");

        // Register tools
        for tool in tools {
            let name = tool.name.clone();
            server = server.tool(Tool::new(&name).description(tool.description.unwrap_or_default()));
        }

        // Register resources metadata (URIs are file:// paths)
        for res in manifests {
            let uri = format!("file://{}", res.path.display());
            server = server.resource(Resource::new(&uri).name("golem.yaml").mime_type("application/yaml"));
        }

        // Handlers
        let handler = ServerHandlerImpl { ctx };

        // Start stdio or SSE server; use SSE via hyper-server on the port
        let addr = ([127, 0, 0, 1], port).into();
        rust_mcp_sdk::server::hyper_server::create_server(server.build(handler)?, addr)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    struct ServerHandlerImpl {
        ctx: Arc<Context>,
    }

    #[async_trait::async_trait]
    impl ServerHandler for ServerHandlerImpl {
        async fn initialize(&self, _params: InitializeRequest) -> anyhow::Result<InitializeResult> {
            Ok(InitializeResult::success())
        }

        async fn tools_list(&self, _params: ToolsListRequest) -> anyhow::Result<ToolsListResult> {
            let tools = discover_tools_from_clap()
                .into_iter()
                .map(|t| Tool::new(&t.name).description(t.description.unwrap_or_default()))
                .collect();
            Ok(ToolsListResult { tools })
        }

        async fn tools_call(&self, params: ToolsCallRequest) -> anyhow::Result<ToolsCallResult> {
            // Expect tool name like `a.b.c` and generic `args` array.
            let name = &params.name;
            let args = params
                .arguments
                .as_object()
                .cloned()
                .unwrap_or_default();

            let mut arg_list: Vec<String> = vec![];
            if let Some(v) = args.get("args") {
                if let Some(arr) = v.as_array() {
                    for a in arr {
                        if let Some(s) = a.as_str() {
                            arg_list.push(s.to_owned());
                        }
                    }
                }
            }

            // Build path segments from tool name
            let segments: Vec<String> = name.split('.').map(|s| s.to_string()).collect();

            // Execute via programmatic invocation to share context and tracing
            let argv = build_tool_argv(&segments, &arg_list);
            let exit = invoke_cli_argv(argv).await;
            let code = match exit { ExitCode::SUCCESS => 0, _ => 1 };

            Ok(ToolsCallResult::success(json!({
                "status": code,
            })))
        }

        async fn resources_list(
            &self,
            _params: ResourcesListRequest,
        ) -> anyhow::Result<ResourcesListResult> {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let manifests = discover_manifest_resources(&cwd);
            let resources = manifests
                .into_iter()
                .map(|r| {
                    let uri = format!("file://{}", r.path.display());
                    Resource::new(&uri).name("golem.yaml").mime_type("application/yaml")
                })
                .collect();
            Ok(ResourcesListResult { resources })
        }

        async fn resources_read(
            &self,
            params: ResourcesReadRequest,
        ) -> anyhow::Result<ResourcesReadResult> {
            let uri = &params.uri;
            let path = if let Some(stripped) = uri.strip_prefix("file://") {
                PathBuf::from(stripped)
            } else {
                PathBuf::from(uri)
            };
            let content = std::fs::read_to_string(&path)?;
            Ok(ResourcesReadResult::success(ResourceContents::new(
                &params.uri,
                "application/yaml",
                content,
            )))
        }
    }
}

/// Starts the MCP server on the given port and runs until shutdown.
pub async fn run_server(ctx: Arc<Context>, port: u16) -> anyhow::Result<()> {
    #[cfg(feature = "mcp-serve")]
    {
        return mcp_server::run(ctx, port).await;
    }

    // Fallback placeholder when feature is not enabled: enumerate and block on Ctrl-C
    let tools = discover_tools_from_clap();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let manifests = discover_manifest_resources(&cwd);

    log::logln(format!(
        "Starting MCP server (placeholder) on port {} with {} tools and {} resources",
        port,
        tools.len(),
        manifests.len()
    ));

    log::logln("Press Ctrl-C to stop.");
    tokio::signal::ctrl_c().await?;
    log::logln("Shutting down serve mode.");

    drop(ctx);
    Ok(())
}

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: Option<String>,
    pub path_segments: Vec<String>,
}

/// Discover CLI tools by traversing the Clap command tree and collecting leaf subcommands.
pub fn discover_tools_from_clap() -> Vec<ToolDefinition> {
    use clap::CommandFactory;
    let command = crate::command::GolemCliCommand::command();
    let mut results = Vec::new();
    let mut path = Vec::<String>::new();
    collect_tools(&mut results, &mut path, &command);
    results
}

fn collect_tools(results: &mut Vec<ToolDefinition>, path: &mut Vec<String>, cmd: &clap::Command) {
    // Skip the root command name; start paths at subcommands
    for sub in cmd.get_subcommands() {
        path.push(sub.get_name().to_string());
        let has_children = sub.get_subcommands().next().is_some();
        if has_children {
            collect_tools(results, path, sub);
        } else {
            // Leaf subcommand; build tool name like `a.b.c`
            let name = path.join(".");
            let description = sub.get_about().map(|s| s.to_string());
            results.push(ToolDefinition {
                name,
                description,
                path_segments: path.clone(),
            });
        }
        path.pop();
    }
}

/// Build argv vector for invoking a tool path with additional arguments.
pub fn build_tool_argv(path_segments: &[String], args: &[String]) -> Vec<OsString> {
    let mut argv: Vec<OsString> = Vec::with_capacity(1 + path_segments.len() + args.len());
    argv.push(OsString::from(command_name()));
    for seg in path_segments {
        argv.push(OsString::from(seg));
    }
    for a in args {
        argv.push(OsString::from(a));
    }
    argv
}

/// Local hooks for invoking the CLI programmatically in serve mode.
struct ServeHooks;
impl crate::command_handler::CommandHandlerHooks for ServeHooks {}

/// Invoke the CLI with the provided argv and return the exit code.
pub async fn invoke_cli_argv<I, T>(argv: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    CommandHandler::handle_args(argv, Arc::new(ServeHooks)).await
}

/// Result of running a CLI sub-process.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub status_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Run the current CLI as a subprocess with the given tool path and args, capturing stdout/stderr.
pub fn run_cli_subprocess(
    path_segments: &[String],
    args: &[String],
    cwd: Option<&Path>,
    envs: Option<&[(String, String)]>,
) -> io::Result<ExecutionResult> {
    let mut cmd = std::process::Command::new(std::env::current_exe()?);
    for seg in path_segments {
        cmd.arg(seg);
    }
    for a in args {
        cmd.arg(a);
    }
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    if let Some(env_pairs) = envs {
        for (k, v) in env_pairs {
            cmd.env(k, v);
        }
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let output = cmd.output()?;
    let status_code = output.status.code().unwrap_or_default();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    Ok(ExecutionResult {
        status_code,
        stdout,
        stderr,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ManifestResource {
    pub path: PathBuf,
}

/// Discover `golem.yaml` files in current dir, ancestors, and one-level children.
pub fn discover_manifest_resources(base: &Path) -> Vec<ManifestResource> {
    let mut found: BTreeSet<PathBuf> = BTreeSet::new();

    // Current directory
    if let Some(p) = find_manifest_in_dir(base) {
        found.insert(p);
    }

    // Ancestors
    for ancestor in base.ancestors() {
        if let Some(p) = find_manifest_in_dir(ancestor) {
            found.insert(p);
        }
    }

    // One-level children (directories only)
    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(p) = find_manifest_in_dir(&path) {
                    found.insert(p);
                }
            }
        }
    }

    found
        .into_iter()
        .map(|path| ManifestResource { path })
        .collect()
}

fn find_manifest_in_dir(dir: &Path) -> Option<PathBuf> {
    let candidate = dir.join("golem.yaml");
    if candidate.is_file() {
        // Canonicalize if possible, otherwise return as-is
        return std::fs::canonicalize(&candidate).ok().or(Some(candidate));
    }
    None
} 