use crate::command::shared_args::{ProjectOptionalFlagArg, UpdateOrRedeployArgs};
use crate::command::GolemCliGlobalFlags;
use crate::command_handler::api::definition::ApiDefinitionCommandHandler;
use crate::command_handler::mcp_server::GolemCliMcpServer;
use crate::log::{McpClient, Output};
use crate::model::api::{ApiDefinitionId, ApiDefinitionVersion};
use crate::model::app::HttpApiDefinitionName;
use console::strip_ansi_codes;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::{schemars, tool, tool_router, Error as CallToolError, Peer, RoleServer};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;

/// Deploy API Definitions and required components
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Deploy {
    /// API definition to deploy, if not specified, all definitions are deployed
    http_api_definition_name: Option<HttpApiDefinitionName>,
    update_or_redeploy: UpdateOrRedeployArgs,
}

/// Retrieves metadata about an existing API definition
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Get {
    project: ProjectOptionalFlagArg,
    /// API definition id
    id: ApiDefinitionId,
    /// Version of the api definition
    version: ApiDefinitionVersion,
}

/// Lists all API definitions
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct List {
    project: ProjectOptionalFlagArg,
    /// API definition id to get all versions. Optional.
    id: Option<ApiDefinitionId>,
}

/// Deletes an existing API definition
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Delete {
    project: ProjectOptionalFlagArg,
    /// API definition id
    id: ApiDefinitionId,
    /// Version of the api definition
    version: ApiDefinitionVersion,
}

#[tool_router(router= tool_router_api_definition, vis="pub")]
impl GolemCliMcpServer {
    #[tool(
        name = "get_api_definition",
        description = "Retrieves metadata about an existing API definition"
    )]
    pub async fn get_api_definition(
        &self,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Get>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(McpClient {
                client,
                tool_name: "get_api_definition".to_owned(),
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ApiDefinitionCommandHandler::new(ctx.into());
                match command_new.cmd_get(req.project, req.id, req.version).await {
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
        name = "list_api_definitions",
        description = "Lists all API definitions"
    )]
    pub async fn list_api_definitions(
        &self,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<List>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(McpClient {
                client,
                tool_name: "list_api_definitions".to_owned(),
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ApiDefinitionCommandHandler::new(ctx.into());
                match command_new.cmd_list(req.project, req.id).await {
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
        name = "delete_api_definition",
        description = "Delete an existing API definition"
    )]
    pub async fn delete_api_definition(
        &self,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Delete>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(McpClient {
                client,
                tool_name: "delete_api_definition".to_owned(),
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ApiDefinitionCommandHandler::new(ctx.into());
                match command_new
                    .cmd_delete(req.project, req.id, req.version)
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
        name = "deploy_api_definitions",
        description = "Deploy API Definitions and required components"
    )]
    pub async fn deploy_api_definitions(
        &self,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Deploy>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(McpClient {
                client,
                tool_name: "deploy_api_definitions".to_owned(),
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ApiDefinitionCommandHandler::new(ctx.into());
                match tokio::task::spawn_blocking(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        command_new
                            .cmd_deploy(req.http_api_definition_name, req.update_or_redeploy)
                            .await
                    })
                })
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
