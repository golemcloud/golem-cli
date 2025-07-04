use crate::command::shared_args::UpdateOrRedeployArgs;
use crate::command::GolemCliGlobalFlags;
use crate::command_handler::api::ApiCommandHandler;
use crate::command_handler::mcp_server::GolemCliMcpServer;
use crate::log::{get_mcp_tool_output, Mcp, Output};
use console::strip_ansi_codes;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content, Meta};
use rmcp::{schemars, tool, tool_router, Error as CallToolError, Peer, RoleServer};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;

pub mod cloud;
pub mod definition;
pub mod deployment;
pub mod security_scheme;

/// Deploy API Definitions and Deployments
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Deploy {
    update_or_redeploy: UpdateOrRedeployArgs,
}

#[tool_router(router= tool_router_api, vis="pub")]
impl GolemCliMcpServer {
    #[tool(
        name = "deploy_api_definitions_and_deployments",
        description = "Deploy API Definitions and Deployments"
    )]
    pub async fn deploy_api_definitions_and_deployments(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Deploy>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "deploy_api_definitions_and_deployments".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ApiCommandHandler::new(ctx.into());
                match tokio::task::spawn_blocking(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async { command_new.cmd_deploy(req.update_or_redeploy).await })
                })
                .await
                {
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
