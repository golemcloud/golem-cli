use crate::command::shared_args::AccountIdOptionalArg;
use crate::command::GolemCliGlobalFlags;
use crate::command_handler::cloud::account::grant::CloudAccountGrantCommandHandler;
use crate::command_handler::mcp_server::GolemCliMcpServer;
use crate::log::{McpClient, Output};
use crate::model::Role;
use console::strip_ansi_codes;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::{schemars, tool, tool_router, Error as CallToolError, Peer, RoleServer};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;

/// Get the roles granted to the account
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Get {
    account_id: AccountIdOptionalArg,
}

/// Grant a new role to the account
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct New {
    account_id: AccountIdOptionalArg,
    /// The role to be granted
    role: Role,
}

/// Remove a role from the account
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Delete {
    account_id: AccountIdOptionalArg,
    /// The role to be deleted
    role: Role,
}

#[tool_router(router= tool_router_cloud_account_grant, vis="pub")]
impl GolemCliMcpServer {
    #[tool(
        name = "create_new_cloud_account_grant",
        description = "Grant a new role to the account"
    )]
    pub async fn create_new_cloud_account_grant(
        &self,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<New>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(McpClient {
                client,
                tool_name: "create_new_cloud_account_grant".to_owned(),
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = CloudAccountGrantCommandHandler::new(ctx.into());
                match command_new
                    .cmd_new(req.account_id.account_id, req.role)
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
        name = "get_cloud_account_grant_roles",
        description = "Get the roles granted to the account"
    )]
    pub async fn get_cloud_account_grant_roles(
        &self,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Get>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(McpClient {
                client,
                tool_name: "get_cloud_account_grant_roles".to_owned(),
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = CloudAccountGrantCommandHandler::new(ctx.into());
                match command_new.cmd_get(req.account_id.account_id).await {
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
        name = "delete_cloud_account_grant_role",
        description = "Remove a role from the account"
    )]
    pub async fn delete_cloud_account_grant_role(
        &self,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Delete>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(McpClient {
                client,
                tool_name: "delete_cloud_account_grant_role".to_owned(),
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = CloudAccountGrantCommandHandler::new(ctx.into());
                match command_new
                    .cmd_delete(req.account_id.account_id, req.role)
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
