use crate::command::shared_args::ProjectArg;
use crate::command::GolemCliGlobalFlags;
use crate::command_handler::api::cloud::domain::ApiCloudDomainCommandHandler;
use crate::command_handler::mcp_server::GolemCliMcpServer;
use crate::log::{get_mcp_tool_output, Mcp, Output};
use console::strip_ansi_codes;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content, Meta};
use rmcp::{schemars, tool, tool_router, ErrorData as CallToolError, Peer, RoleServer};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;

/// Retrieves metadata about an existing domain
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Get {
    project: ProjectArg,
}

/// Add new domain
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct New {
    project: ProjectArg,
    /// Domain name
    domain_name: String,
}

/// Delete an existing domain
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Delete {
    project: ProjectArg,
    /// Domain name
    domain_name: String,
}

#[tool_router(router= tool_router_api_cloud_domain, vis="pub")]
impl GolemCliMcpServer {
    #[tool(
        name = "get_domain",
        description = "Retrieves metadata about an existing domain"
    )]
    pub async fn get_domain(
        &self,
        _meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Get>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "get_domain".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ApiCloudDomainCommandHandler::new(ctx.into());
                match command_new.cmd_get(req.project.project).await {
                    Ok(_) => Ok(CallToolResult {
                        content: get_mcp_tool_output()
                            .into_iter()
                            .map(Content::text)
                            .collect(),

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

    #[tool(name = "delete_domain", description = "Delete an existing domain")]
    pub async fn delete_domain(
        &self,
        _meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Delete>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "delete_domain".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ApiCloudDomainCommandHandler::new(ctx.into());
                match command_new
                    .cmd_delete(req.project.project, req.domain_name)
                    .await
                {
                    Ok(_) => Ok(CallToolResult {
                        content: get_mcp_tool_output()
                            .into_iter()
                            .map(Content::text)
                            .collect(),

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

    #[tool(name = "add_new_domain", description = "Add new domain")]
    pub async fn add_new_domain(
        &self,
        _meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<New>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "add_new_domain".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ApiCloudDomainCommandHandler::new(ctx.into());
                match command_new
                    .cmd_new(req.project.project, req.domain_name)
                    .await
                {
                    Ok(_) => Ok(CallToolResult {
                        content: get_mcp_tool_output()
                            .into_iter()
                            .map(Content::text)
                            .collect(),

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
