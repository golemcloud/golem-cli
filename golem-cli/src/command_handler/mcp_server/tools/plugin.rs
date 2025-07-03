use crate::command::shared_args::PluginScopeArgs;
use crate::command::GolemCliGlobalFlags;
use crate::command_handler::mcp_server::GolemCliMcpServer;
use crate::command_handler::plugin::PluginCommandHandler;
use crate::log::{Mcp, Output};
use crate::model::PathBufOrStdin;
use console::strip_ansi_codes;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content, Meta};
use rmcp::{schemars, tool, tool_router, Error as CallToolError, Peer, RoleServer};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;

/// List component for the select scope
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct List {
    /// The scope to list components from
    scope: PluginScopeArgs,
}

/// Get information about a registered plugin
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Get {
    /// Plugin name
    plugin_name: String,
    /// Plugin version
    version: String,
}

/// Register a new plugin
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Register {
    scope: PluginScopeArgs,
    /// Path to the plugin manifest JSON or '-' to use STDIN
    manifest: PathBufOrStdin,
}

/// Unregister a plugin
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Unregister {
    /// Plugin name
    plugin_name: String,
    /// Plugin version
    version: String,
}

#[tool_router(router= tool_router_plugin, vis="pub")]
impl GolemCliMcpServer {
    #[tool(
        name = "list_components_for_scope",
        description = "List component for the select scope"
    )]
    pub async fn list_components_for_scope(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<List>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "list_components_for_scope".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = PluginCommandHandler::new(ctx.into());

                match command_new.cmd_list(req.scope).await {
                    Ok(_) => Ok(CallToolResult {
                        content: vec![Content::text("Success")],
                        is_error: None,
                    }),
                    Err(e) => Ok(CallToolResult {
                        content: vec![Content::text(strip_ansi_codes(&e.to_string()).to_string())],
                        is_error: Some(true),
                    }),
                }
            }
            Err(e) => Ok(CallToolResult {
                content: vec![Content::text(strip_ansi_codes(&e.to_string()).to_string())],
                is_error: Some(true),
            }),
        }
    }

    #[tool(
        name = "get_plugin",
        description = "Get information about a registered plugin"
    )]
    pub async fn get_plugin(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Unregister>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "get_plugin".to_string(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = PluginCommandHandler::new(ctx.into());

                match command_new.cmd_get(req.plugin_name, req.version).await {
                    Ok(_) => Ok(CallToolResult {
                        content: vec![Content::text("Success")],
                        is_error: None,
                    }),
                    Err(e) => Ok(CallToolResult {
                        content: vec![Content::text(strip_ansi_codes(&e.to_string()).to_string())],
                        is_error: Some(true),
                    }),
                }
            }
            Err(e) => Ok(CallToolResult {
                content: vec![Content::text(strip_ansi_codes(&e.to_string()).to_string())],
                is_error: Some(true),
            }),
        }
    }

    #[tool(name = "register_plugin", description = "Register a new plugin")]
    pub async fn register_plugin(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Register>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "register_plugin".to_string(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = PluginCommandHandler::new(ctx.into());

                match command_new.cmd_register(req.scope, req.manifest).await {
                    Ok(_) => Ok(CallToolResult {
                        content: vec![Content::text("Success")],
                        is_error: None,
                    }),
                    Err(e) => Ok(CallToolResult {
                        content: vec![Content::text(strip_ansi_codes(&e.to_string()).to_string())],
                        is_error: Some(true),
                    }),
                }
            }
            Err(e) => Ok(CallToolResult {
                content: vec![Content::text(strip_ansi_codes(&e.to_string()).to_string())],
                is_error: Some(true),
            }),
        }
    }

    #[tool(name = "unregister_plugin", description = "Unregister a new plugin")]
    pub async fn unregister_plugin(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Unregister>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "unregister_plugin".to_string(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = PluginCommandHandler::new(ctx.into());

                match command_new
                    .cmd_unregister(req.plugin_name, req.version)
                    .await
                {
                    Ok(_) => Ok(CallToolResult {
                        content: vec![Content::text("Success")],
                        is_error: None,
                    }),
                    Err(e) => Ok(CallToolResult {
                        content: vec![Content::text(strip_ansi_codes(&e.to_string()).to_string())],
                        is_error: Some(true),
                    }),
                }
            }
            Err(e) => Ok(CallToolResult {
                content: vec![Content::text(strip_ansi_codes(&e.to_string()).to_string())],
                is_error: Some(true),
            }),
        }
    }
}
