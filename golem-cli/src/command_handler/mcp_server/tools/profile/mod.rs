use crate::command::GolemCliGlobalFlags;
use crate::command_handler::mcp_server::GolemCliMcpServer;
use crate::command_handler::profile::ProfileCommandHandler;
use crate::config::ProfileName;
use crate::log::{get_mcp_tool_output, Mcp, Output};
use crate::model::Format;
use console::strip_ansi_codes;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content, Meta};
use rmcp::{schemars, tool, tool_router, Error as CallToolError, Peer, RoleServer};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;
use url::Url;
// use uuid::Uuid;

pub mod config;

/// Create new global profile, call without <PROFILE_NAME> for interactive setup
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct New {
    /// Name of the newly created profile
    name: Option<ProfileName>,
    /// Switch to the profile after creation
    set_active: bool,
    /// URL of Golem Component service
    #[schemars(with = "String")]
    component_url: Option<Url>,
    /// URL of Golem Worker service, if not provided defaults to component-url
    #[schemars(with = "String")]
    worker_url: Option<Url>,
    /// URL of Golem Cloud service, if not provided defaults to component-url
    #[schemars(with = "String")]
    cloud_url: Option<Url>,
    /// Default output format
    default_format: Format,
    
    /// Token to use for authenticating against Golem. If not provided an OAuth2 flow will be performed when authentication is needed for the first time.
    // #[schemars(with = "String")]
    // static_token: Option<Uuid>, disabled

    /// Accept invalid certificates.
    ///
    /// Disables certificate validation.
    /// Warning! Any certificate will be trusted for use.
    /// This includes expired certificates.
    /// This introduces significant vulnerabilities, and should only be used as a last resort.
    allow_insecure: bool,
}

/// List global profiles
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct List {}

/// Set the active global default profile
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Switch {
    /// Profile name to switch to
    profile_name: ProfileName,
}

/// Show global profile details
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Get {
    /// Name of profile to show, shows active profile if not specified.
    profile_name: Option<ProfileName>,
}

/// Remove global profile
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Delete {
    /// Profile name to delete
    profile_name: ProfileName,
}

#[tool_router(router= tool_router_profile, vis="pub")]
impl GolemCliMcpServer {
    #[tool(
        name = "create_new_global_profile",
        description = "Create new global profile"
    )]
    pub async fn create_new_global_profile(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<New>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "create_new_global_profile".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ProfileCommandHandler::new(ctx.into());
                match command_new.cmd_new(
                    req.name,
                    req.set_active,
                    req.component_url,
                    req.worker_url,
                    req.cloud_url,
                    req.default_format,
                    req.allow_insecure,
                    None // req.static_token, not good to allow in mcp, so oauth flow will take place
                ) {
                    Ok(_) => Ok(CallToolResult {
                        content: get_mcp_tool_output().into_iter().map(Content::text).collect(),

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

    #[tool(name = "list_global_profiles", description = "List global profiles")]
    pub async fn list_global_profiles(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(_): Parameters<List>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "list_global_profiles".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ProfileCommandHandler::new(ctx.into());
                match command_new.cmd_list() {
                    Ok(_) => Ok(CallToolResult {
                        content: get_mcp_tool_output().into_iter().map(Content::text).collect(),

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
        name = "show_global_profile_details",
        description = "Show global profile details"
    )]
    pub async fn show_global_profile_details(
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
                tool_name: "Show_global_profile_details".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ProfileCommandHandler::new(ctx.into());
                match command_new.cmd_get(req.profile_name) {
                    Ok(_) => Ok(CallToolResult {
                        content: get_mcp_tool_output().into_iter().map(Content::text).collect(),

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

    #[tool(name = "remove_profile", description = "Remove global profile")]
    pub async fn remove_profile(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Delete>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "remove_profile".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ProfileCommandHandler::new(ctx.into());
                match command_new.cmd_delete(req.profile_name) {
                    Ok(_) => Ok(CallToolResult {
                        content: get_mcp_tool_output().into_iter().map(Content::text).collect(),

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
        name = "switch_profile",
        description = "Set the active global default profile"
    )]
    pub async fn switch_profile(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Switch>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "switch_profile".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ProfileCommandHandler::new(ctx.into());
                match command_new.cmd_switch(req.profile_name) {
                    Ok(_) => Ok(CallToolResult {
                        content: get_mcp_tool_output().into_iter().map(Content::text).collect(),

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
