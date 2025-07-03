use crate::command::shared_args::ProjectArg;
use crate::command::GolemCliGlobalFlags;
use crate::command_handler::api::cloud::certificate::ApiCloudCertificateCommandHandler;
use crate::command_handler::mcp_server::GolemCliMcpServer;
use crate::log::{Mcp, Output};
use crate::model::PathBufOrStdin;
use console::strip_ansi_codes;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content, Meta};
use rmcp::{schemars, tool, Error as CallToolError, Peer, RoleServer};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;
use uuid::Uuid;

/// Retrieves metadata about an existing certificate
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Get {
    project: ProjectArg,
    /// Certificate ID
    #[schemars(with = "String")]
    certificate_id: Option<Uuid>,
}

/// Create new certificate
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct New {
    project: ProjectArg,
    /// Domain name
    domain_name: String,
    /// Certificate
    certificate_body: PathBufOrStdin,
    /// Certificate private key
    certificate_private_key: PathBufOrStdin,
}

/// Delete an existing certificate
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Delete {
    project: ProjectArg,
    /// Certificate ID
    #[schemars(with = "String")]
    certificate_id: Uuid,
}

// #[tool_router(router= tool_router_api_cloud_certificate, vis="pub")] // disabled
impl GolemCliMcpServer {
    #[tool(
        name = "get_certificate",
        description = "Retrieves metadata about an existing certificate"
    )]
    pub async fn get_certificate(
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
                tool_name: "get_certificate".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ApiCloudCertificateCommandHandler::new(ctx.into());
                match command_new
                    .cmd_get(req.project.project, req.certificate_id)
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
        name = "delete_certificate",
        description = "Delete an existing certificate"
    )]
    pub async fn delete_certificate(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Delete>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "delete_certificate".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ApiCloudCertificateCommandHandler::new(ctx.into());
                match command_new
                    .cmd_delete(req.project.project, req.certificate_id)
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

    #[tool(name = "add_new_certificate", description = "Create new certificate")]
    pub async fn add_new_certificate(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<New>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "add_new_Certificate".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ApiCloudCertificateCommandHandler::new(ctx.into());
                match command_new
                    .cmd_new(
                        req.project.project,
                        req.domain_name,
                        req.certificate_body,
                        req.certificate_private_key,
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
