use console::strip_ansi_codes;
use rmcp::{
    handler::server::tool::Parameters,
    model::{CallToolResult, Content, Meta},
    service::RequestContext,
    tool, tool_router, Error as CallToolError, Peer, RoleServer, ServerHandler,
};
use serde::{Deserialize, Serialize};

use crate::{
    command_handler::mcp_server::GolemCliMcpServer,
};
use std::future::Future;

pub mod api;
pub mod app;
pub mod cloud;
pub mod component;
pub mod plugin;
pub mod profile;
pub mod repl;
pub mod worker;

/// List Tools
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListTools {}

/// Get Tool info
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetTool {
    tool_name: String,
}

#[tool_router(router= tool_router_list_tools, vis="pub")]
impl GolemCliMcpServer {
    #[tool(
        name = "list_golem_mcp_server_tools",
        description = "tool to list tools this golem-cli mcp server"
    )]
    pub async fn list_golem_mcp_server_tools(
        &self,
        _meta: Meta,
        context: RequestContext<RoleServer>,
        _client: Peer<RoleServer>,
        Parameters(_): Parameters<ListTools>,
    ) -> Result<CallToolResult, CallToolError> {
        let tools = self.list_tools(None, context).await;
        match tools {
            Ok(tools_result) => Ok(CallToolResult {
                content: tools_result
                    .tools
                    .into_iter()
                    .map(|tool| Content::text(tool.schema_as_json_value().to_string()))
                    .collect(),
                is_error: None,
            }),
            Err(e) => Ok(CallToolResult {
                content: vec![Content::text(strip_ansi_codes(&e.to_string()).to_string())],
                is_error: Some(true),
            }),
        }
    }

    #[tool(
        name = "get_golem_mcp_server_tool_info",
        description = "Get golem mcp server tool info"
    )]
    pub async fn get_golem_mcp_server_tool_info(
        &self,
        _meta: Meta,
        context: RequestContext<RoleServer>,
        _client: Peer<RoleServer>,
        Parameters(req): Parameters<GetTool>,
    ) -> Result<CallToolResult, CallToolError> {
        let tools = self.list_tools(None, context).await;
        match tools {
            Ok(tools_result) => Ok(CallToolResult {
                content: tools_result
                    .tools
                    .into_iter()
                    .filter(|tool| strip_ansi_codes(&tool.name).to_string() == req.tool_name)
                    .map(|tool| Content::text(tool.schema_as_json_value().to_string()))
                    .collect(),
                is_error: None,
            }),
            Err(e) => Ok(CallToolResult {
                content: vec![Content::text(strip_ansi_codes(&e.to_string()).to_string())],
                is_error: Some(true),
            }),
        }
    }
}
