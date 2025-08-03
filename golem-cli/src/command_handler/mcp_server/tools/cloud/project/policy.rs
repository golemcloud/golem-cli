use crate::command::GolemCliGlobalFlags;
use crate::command_handler::cloud::project::policy::CloudProjectPolicyCommandHandler;
use crate::command_handler::mcp_server::GolemCliMcpServer;
use crate::log::{get_mcp_tool_output, Mcp, Output};
// use crate::model::ProjectPermission;
use crate::model::ProjectPolicyId;
use console::strip_ansi_codes;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content, Meta};
use rmcp::{schemars, tool, tool_router, ErrorData as CallToolError, Peer, RoleServer};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;

// /// Creates a new project sharing policy
// #[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
// pub struct New {
//     /// Name of the policy
//     policy_name: String,
//     /// List of actions allowed by the policy
//     actions: Vec<ProjectPermission>,
// }

/// Gets the existing project sharing policies
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Get {
    /// Project policy ID
    #[schemars(with = "String")]
    policy_id: ProjectPolicyId,
}

#[tool_router(router= tool_router_cloud_project_policy, vis="pub")]
impl GolemCliMcpServer {
    // Todo: need changes in golem repo

    // #[tool(
    //     name = "create_new_project_policy",
    //     description = "Creates a new project sharing policy"
    // )]
    // pub async fn create_new_project_policy(
    //     &self,
    //     _meta: Meta,
    //     client: Peer<RoleServer>,
    //     Parameters(req): Parameters<New>,
    // ) -> Result<CallToolResult, CallToolError> {
    //     let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

    //     match crate::context::Context::new(
    //         GolemCliGlobalFlags::default(),
    //         Some(Output::Mcp(Mcp {
    //             client,
    //             tool_name: "create_new_project_policy".to_owned(),
    //             // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
    //         })),
    //         start_local_server_yes,
    //         Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
    //     )
    //     .await
    //     {
    //         Ok(ctx) => {
    //             let command_new = CloudProjectPolicyCommandHandler::new(ctx.into());
    //             match command_new.cmd_new(req.policy_name, req.actions).await {
    //                 Ok(_) => Ok(CallToolResult {
    //                     content: get_mcp_tool_output().into_iter().map(Content::text).collect(),

    //                     is_error: None,
    //                 }),
    //                 Err(e) => Ok(CallToolResult {
    //                     content: vec![Content::text(strip_ansi_codes(&e.to_string()).to_string())],
    //                     is_error: Some(true),
    //                 }),
    //             }
    //         }
    //         Err(e) => Ok(CallToolResult {
    //             content: vec![Content::text(strip_ansi_codes(&e.to_string()).to_string())],
    //             is_error: Some(true),
    //         }),
    //     }
    // }

    #[tool(
        name = "get_project_policies",
        description = "Gets the existing project sharing policies"
    )]
    pub async fn get_project_policies(
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
                tool_name: "get_project_policies".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = CloudProjectPolicyCommandHandler::new(ctx.into());
                match command_new.cmd_get(req.policy_id).await {
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
