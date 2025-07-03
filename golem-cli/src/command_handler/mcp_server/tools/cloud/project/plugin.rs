use crate::command::shared_args::ProjectArg;
use crate::command::GolemCliGlobalFlags;
use crate::command_handler::cloud::project::plugin::CloudProjectPluginCommandHandler;
use crate::command_handler::mcp_server::GolemCliMcpServer;
use crate::log::{Mcp, Output};
use console::strip_ansi_codes;
use golem_common::model::PluginInstallationId;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content, Meta};
use rmcp::{schemars, tool, tool_router, Error as CallToolError, Peer, RoleServer};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;

/// Install a plugin for a project
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Install {
    project: ProjectArg,
    /// The plugin to install
    plugin_name: String,
    /// The version of the plugin to install
    plugin_version: String,
    /// Priority of the plugin - largest priority is applied first
    priority: i32,
    /// List of parameters (key-value pairs) passed to the plugin
    param: Vec<(String, String)>,
}

/// Get the installed plugins for the project
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Get {
    project: ProjectArg,
    /* TODO: Missing from HTTP API
    /// The version of the component
    version: Option<u64>,
    */
}

/// Update project plugin
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Update {
    project: ProjectArg,
    /// Installation id of the plugin to update
    #[schemars(with = "String")]
    plugin_installation_id: PluginInstallationId,
    /// Updated priority of the plugin - largest priority is applied first
    priority: i32,
    /// Updated list of parameters (key-value pairs) passed to the plugin
    param: Vec<(String, String)>,
}

/// Uninstall a plugin for selected component
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Uninstall {
    project: ProjectArg,
    /// Installation id of the plugin to uninstall
    #[schemars(with = "String")]
    plugin_installation_id: PluginInstallationId,
}

#[tool_router(router= tool_router_cloud_project_plugin, vis="pub")]
impl GolemCliMcpServer {
    #[tool(
        name = "install_project_plugin",
        description = "Install a plugin for a project"
    )]
    pub async fn install_project_plugin(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Install>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "install_project_plugin".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = CloudProjectPluginCommandHandler::new(ctx.into());
                match command_new
                    .cmd_install(
                        req.project.project,
                        req.plugin_name,
                        req.plugin_version,
                        req.priority,
                        req.param,
                    )
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

    #[tool(
        name = "uninstall_project_plugin",
        description = "Uninstall a plugin for selected component"
    )]
    pub async fn uninstall_project_plugin(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Uninstall>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "uninstall_project_plugin".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = CloudProjectPluginCommandHandler::new(ctx.into());
                match command_new
                    .cmd_uninstall(req.project.project, req.plugin_installation_id)
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

    #[tool(
        name = "get_cloud_project_plugins",
        description = "Get the installed plugins for the project"
    )]
    pub async fn get_cloud_project_plugins(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Get>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "get_cloud_project_plugins".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = CloudProjectPluginCommandHandler::new(ctx.into());
                match command_new.cmd_get(req.project.project).await {
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

    #[tool(name = "update_project_plugin", description = "Update project plugin")]
    pub async fn update_project_plugin(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Update>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "update_project_plugin".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = CloudProjectPluginCommandHandler::new(ctx.into());
                match command_new
                    .cmd_update(
                        req.project.project,
                        req.plugin_installation_id,
                        req.priority,
                        req.param,
                    )
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
