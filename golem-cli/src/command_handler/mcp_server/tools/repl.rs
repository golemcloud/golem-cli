use crate::command::shared_args::ComponentOptionalComponentName;
use crate::command::GolemCliGlobalFlags;
use crate::command_handler::mcp_server::GolemCliMcpServer;
use crate::command_handler::rib_repl::RibReplHandler;
use crate::log::{get_mcp_tool_output, Mcp, Output};
use console::strip_ansi_codes;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content, Meta};
use rmcp::{schemars, tool, tool_router, Error as CallToolError, Peer, RoleServer};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;

/// Start Rib REPL for a selected component
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Repl {
    component_name: ComponentOptionalComponentName,
    /// Optional component version to use, defaults to latest component version
    version: Option<u64>,
}

#[tool_router(router= tool_router_repl, vis="pub")]
impl GolemCliMcpServer {
    #[tool(
        name = "start_rib_repl",
        description = "Start Rib REPL for a selected component"
    )]
    pub async fn start_rib_repl(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Repl>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "start_rib_repl".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = RibReplHandler::new(ctx.into());
                match tokio::task::spawn_blocking(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        command_new
                            .cmd_repl(req.component_name.component_name, req.version)
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
