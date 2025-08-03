use crate::command::shared_args::{ProjectOptionalFlagArg, UpdateOrRedeployArgs};
use crate::command::GolemCliGlobalFlags;
use crate::command_handler::api::deployment::ApiDeploymentCommandHandler;
use crate::command_handler::mcp_server::GolemCliMcpServer;
use crate::log::{get_mcp_tool_output, Mcp, Output};
use crate::model::api::ApiDefinitionId;
use console::strip_ansi_codes;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content, Meta};
use rmcp::{schemars, tool, tool_router, ErrorData as CallToolError, Peer, RoleServer};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;

/// Deploy API Deployments
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Deploy {
    /// Host or site to deploy, if not defined, all deployments will be deployed
    host_or_site: Option<String>,
    update_or_redeploy: UpdateOrRedeployArgs,
}

/// Get API deployment
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Get {
    project: ProjectOptionalFlagArg,
    /// Deployment site
    site: String,
}

/// List API deployment for API definition
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct List {
    project: ProjectOptionalFlagArg,
    /// API definition id
    definition: Option<ApiDefinitionId>,
}

/// Delete api deployment
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Delete {
    project: ProjectOptionalFlagArg,
    /// Deployment site
    site: String,
}

#[tool_router(router= tool_router_api_deployment, vis="pub")]
impl GolemCliMcpServer {
    #[tool(
        name = "get_api_deployment",
        description = "Retrieves metadata about an existing API deployment"
    )]
    pub async fn get_api_deployment(
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
                tool_name: "get_api_deployment".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ApiDeploymentCommandHandler::new(ctx.into());
                match command_new.cmd_get(req.project, req.site).await {
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

    #[tool(
        name = "list_api_deployment",
        description = "List API deployment for API definition"
    )]
    pub async fn list_api_deployment(
        &self,
        _meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<List>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "list_api_deployment".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ApiDeploymentCommandHandler::new(ctx.into());
                match command_new.cmd_list(req.project, req.definition).await {
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

    #[tool(name = "delete_api_deployment", description = "Delete api deployment")]
    pub async fn delete_api_deployment(
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
                tool_name: "delete_api_deployment".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ApiDeploymentCommandHandler::new(ctx.into());
                match command_new.cmd_delete(req.project, req.site).await {
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

    #[tool(
        name = "deploy_api_deployments",
        description = "Deploy API Deployments"
    )]
    pub async fn deploy_api_deployments(
        &self,
        _meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Deploy>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "deploy_api_deployments".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ApiDeploymentCommandHandler::new(ctx.into());
                match tokio::task::spawn_blocking(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        command_new
                            .cmd_deploy(req.host_or_site, req.update_or_redeploy)
                            .await
                    })
                })
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
