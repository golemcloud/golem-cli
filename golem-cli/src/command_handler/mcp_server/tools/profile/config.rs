use crate::command::GolemCliGlobalFlags;
use crate::command_handler::mcp_server::GolemCliMcpServer;
use crate::command_handler::profile::config::ProfileConfigCommandHandler;
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

/// Set default output format for the requested profile
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SetFormat {
    /// Profile name
    profile_name: ProfileName,
    /// CLI output format
    format: Format,
}

#[tool_router(router= tool_router_profile_config, vis="pub")]
impl GolemCliMcpServer {
    #[tool(
        name = "set_default_output_format_profile",
        description = "Set default output format for the requested profile"
    )]
    pub async fn set_default_output_format_profile(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<SetFormat>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "set_default_output_format_profile".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ProfileConfigCommandHandler::new(ctx.into());
                match command_new.cmd_set_format(req.profile_name, req.format) {
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
