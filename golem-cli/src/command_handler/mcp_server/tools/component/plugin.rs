use crate::command::shared_args::ComponentOptionalComponentName;
use crate::command::GolemCliGlobalFlags;
use crate::command_handler::component::plugin::ComponentPluginCommandHandler;
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

/// Install a plugin for selected component
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Install {
    component_name: ComponentOptionalComponentName,
    /// The plugin to install
    plugin_name: String,
    /// The version of the plugin to install
    plugin_version: String,
    /// Priority of the plugin - largest priority is applied first
    priority: i32,
    /// List of parameters (key-value pairs) passed to the plugin
    param: Vec<(String, String)>,
}

/// Get the installed plugins of the component
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]

pub struct Get {
    component_name: ComponentOptionalComponentName,
    /// The version of the component
    version: Option<u64>,
}

/// Update component plugin
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Update {
    /// The component to update the plugin for
    component_name: ComponentOptionalComponentName,
    /// Installation id of the plugin to update
    #[schemars(with = "String")]
    installation_id: PluginInstallationId,
    /// Updated priority of the plugin - largest priority is applied first
    priority: i32,
    /// Updated list of parameters (key-value pairs) passed to the plugin
    param: Vec<(String, String)>,
}

/// Uninstall a plugin for selected component
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Uninstall {
    /// The component to uninstall the plugin from
    component_name: ComponentOptionalComponentName,
    /// Installation id of the plugin to uninstall
    #[schemars(with = "String")]
    installation_id: PluginInstallationId,
}

#[tool_router(router= tool_router_component_plugin, vis="pub")]
impl GolemCliMcpServer {
    #[tool(
        name = "install_component_plugin",
        description = "Install plugin in a golem app Component"
    )]
    pub async fn install_component_plugin(
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
                tool_name: "install_component_plugin".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ComponentPluginCommandHandler::new(ctx.into());
                match command_new
                    .cmd_install(
                        req.component_name.component_name,
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
        name = "get_component_plugin",
        description = "Get plugin in a golem app Component"
    )]
    pub async fn get_component_plugin(
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
                tool_name: "get_component_plugin".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ComponentPluginCommandHandler::new(ctx.into());
                match command_new
                    .cmd_get(req.component_name.component_name, req.version)
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
        name = "uninstall_component_plugin",
        description = "UnInstall plugin in a golem app Component"
    )]
    pub async fn uninstall_component_plugin(
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
                tool_name: "uninstall_component_plugin".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ComponentPluginCommandHandler::new(ctx.into());
                match command_new
                    .cmd_uninstall(req.component_name.component_name, req.installation_id)
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
        name = "update_component_plugin",
        description = "Update plugin in a golem app Component"
    )]
    pub async fn update_component_plugin(
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
                tool_name: "update_component_plugin".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ComponentPluginCommandHandler::new(ctx.into());
                match command_new
                    .cmd_update(
                        req.component_name.component_name,
                        req.installation_id,
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
