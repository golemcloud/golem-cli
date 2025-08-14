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
	use rust_mcp_sdk::mcp_server::{hyper_server, ServerHandler, HyperServerOptions};
	use rust_mcp_sdk::McpServer;
	use rust_mcp_sdk::schema::{
		InitializeResult, ServerCapabilities, ServerCapabilitiesTools, Implementation, LATEST_PROTOCOL_VERSION,
		ListToolsRequest, ListToolsResult, CallToolRequest, CallToolResult, RpcError,
	};
	use rust_mcp_sdk::schema::schema_utils::CallToolError;

	pub async fn run(_ctx: Arc<Context>, port: u16) -> anyhow::Result<()> {
		// Define server details and capabilities
		let server_details = InitializeResult {
			server_info: Implementation {
				name: "golem-cli".to_string(),
				version: env!("CARGO_PKG_VERSION").to_string(),
				title: Some("Golem CLI MCP Server".to_string()),
			},
			capabilities: ServerCapabilities {
				tools: Some(ServerCapabilitiesTools { list_changed: None }),
				..Default::default()
			},
			meta: None,
			instructions: Some("Golem CLI MCP Server".to_string()),
			protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
		};

		// Handlers
		let handler = ServerHandlerImpl {};

		// Start SSE server via HyperServer
		let server = hyper_server::create_server(
			server_details,
			handler,
			HyperServerOptions {
				host: "127.0.0.1".to_string(),
				port,
				custom_sse_endpoint: Some("/sse".to_string()),
				enable_json_response: Some(true),
				custom_streamable_http_endpoint: Some("/mcp".to_string()),
				..Default::default()
			},
		);
		server.start().await.map_err(|e| anyhow::anyhow!(format!("{}", e)))
	}

	struct ServerHandlerImpl;

	#[async_trait::async_trait]
	impl ServerHandler for ServerHandlerImpl {
		async fn handle_list_tools_request(&self, _request: ListToolsRequest, _runtime: &dyn McpServer) -> Result<ListToolsResult, RpcError> {
			let tools = discover_tools_from_clap();
			let mut results: Vec<rust_mcp_sdk::schema::Tool> = Vec::with_capacity(tools.len());
			for t in tools {
				let mut props = std::collections::HashMap::new();
				let mut arg_schema = ::serde_json::Map::new();
				arg_schema.insert("type".to_string(), serde_json::Value::String("array".to_string()));
				arg_schema.insert("items".to_string(), serde_json::json!({"type": "string"}));
				props.insert("args".to_string(), arg_schema);
				let input = rust_mcp_sdk::schema::ToolInputSchema::new(vec![], Some(props));
				results.push(rust_mcp_sdk::schema::Tool {
					annotations: None,
					description: t.description.clone(),
					input_schema: input,
					meta: None,
					name: t.name,
					output_schema: None,
					title: None,
				});
			}
			Ok(ListToolsResult { tools: results, meta: None, next_cursor: None })
		}

		async fn handle_call_tool_request(&self, request: CallToolRequest, _runtime: &dyn McpServer) -> Result<CallToolResult, CallToolError> {
			let name = &request.params.name;
			let args_obj = request.params.arguments.clone().unwrap_or_default();
			let mut arg_list: Vec<String> = vec![];
			if let Some(v) = args_obj.get("args") {
				if let Some(arr) = v.as_array() {
					for a in arr {
						if let Some(s) = a.as_str() { arg_list.push(s.to_owned()); }
					}
				}
			}
			let segments: Vec<String> = name.split('.').map(|s| s.to_string()).collect();

			let exec = match run_cli_subprocess(&segments, &arg_list, None, None) {
				Ok(r) => r,
				Err(e) => {
					let content = rust_mcp_sdk::schema::TextContent::new(format!("spawn error: {}", e), None, None);
					return Ok(CallToolResult {
						content: vec![content.into()],
						meta: None,
						is_error: Some(true),
						structured_content: None,
					});
				}
			};

			let mut obj = ::serde_json::Map::new();
			obj.insert("status".to_string(), ::serde_json::Value::Number(exec.status_code.into()));
			obj.insert("stdout".to_string(), ::serde_json::Value::String(exec.stdout));
			obj.insert("stderr".to_string(), ::serde_json::Value::String(exec.stderr));

			let content = rust_mcp_sdk::schema::TextContent::new("ok".to_string(), None, None);
			Ok(CallToolResult {
				content: vec![content.into()],
				meta: None,
				is_error: None,
				structured_content: Some(obj),
			})
		}

		async fn handle_list_resources_request(&self, _request: rust_mcp_sdk::schema::ListResourcesRequest, _runtime: &dyn McpServer) -> Result<rust_mcp_sdk::schema::ListResourcesResult, RpcError> {
			let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
			let manifests = discover_manifest_resources(&cwd);
			let resources: Vec<rust_mcp_sdk::schema::Resource> = manifests
				.into_iter()
				.map(|r| {
					let uri = format!("file://{}", r.path.display());
					rust_mcp_sdk::schema::Resource {
						annotations: None,
						description: None,
						meta: None,
						mime_type: Some("application/yaml".to_string()),
						name: "golem.yaml".to_string(),
						size: None,
						title: None,
						uri,
					}
				})
				.collect();
			Ok(rust_mcp_sdk::schema::ListResourcesResult { resources, meta: None, next_cursor: None })
		}

		async fn handle_read_resource_request(&self, request: rust_mcp_sdk::schema::ReadResourceRequest, _runtime: &dyn McpServer) -> Result<rust_mcp_sdk::schema::ReadResourceResult, RpcError> {
			let uri = &request.params.uri;
			let path = if let Some(stripped) = uri.strip_prefix("file://") { PathBuf::from(stripped) } else { PathBuf::from(uri) };
			let content = std::fs::read_to_string(&path)
				.map_err(|e| RpcError::internal_error().with_message(format!("read error: {}", e)))?;
			let text = rust_mcp_sdk::schema::TextResourceContents {
				meta: None,
				mime_type: Some("application/yaml".to_string()),
				text: content,
				uri: uri.clone(),
			};
			Ok(rust_mcp_sdk::schema::ReadResourceResult { contents: vec![text.into()], meta: None })
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