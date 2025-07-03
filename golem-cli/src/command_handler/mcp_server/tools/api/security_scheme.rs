use crate::command::shared_args::ProjectOptionalFlagArg;
use crate::command::GolemCliGlobalFlags;
use crate::command_handler::api::security_scheme::ApiSecuritySchemeCommandHandler;
use crate::command_handler::mcp_server::GolemCliMcpServer;
use crate::log::{Mcp, Output};
use crate::model::api::IdentityProviderType;
use console::strip_ansi_codes;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content, Meta};
use rmcp::{schemars, tool, Error as CallToolError, Peer, RoleServer};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;

/// Create API Security Scheme
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Create {
    project: ProjectOptionalFlagArg,
    /// Security Scheme ID
    security_scheme_id: String,
    /// Security Scheme provider (Google, Facebook, Gitlab, Microsoft)
    provider_type: IdentityProviderType,
    /// Security Scheme client ID
    client_id: String,
    /// Security Scheme client secret
    client_secret: String,
    /// Security Scheme Scopes, can be defined multiple times
    scope: Vec<String>,
    /// Security Scheme redirect URL
    redirect_url: String,
}

/// Get API security
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Get {
    project: ProjectOptionalFlagArg,
    /// Security Scheme ID
    security_scheme_id: String,
}

// #[tool_router(router= tool_router_api_security_scheme, vis="pub")] // disabled
impl GolemCliMcpServer {
    #[tool(name = "get_api_security_scheme", description = "Get API security")]
    pub async fn get_api_security_scheme(
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
                tool_name: "get_api_security_scheme".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ApiSecuritySchemeCommandHandler::new(ctx.into());
                match command_new
                    .cmd_get(req.project, req.security_scheme_id)
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
        name = "create_api_security_scheme",
        description = "Create API Security Scheme"
    )]
    pub async fn create_api_security_scheme(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Create>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "create_api_security_scheme".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ApiSecuritySchemeCommandHandler::new(ctx.into());
                match command_new
                    .cmd_create(
                        req.project,
                        req.security_scheme_id,
                        req.provider_type,
                        req.client_id,
                        req.client_secret,
                        req.scope,
                        req.redirect_url,
                    )
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
