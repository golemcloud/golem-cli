use crate::command::shared_args::{
    AppOptionalComponentNames, BuildArgs, ForceBuildArg, UpdateOrRedeployArgs,
};
use crate::command::GolemCliGlobalFlags;
use crate::command_handler::app::AppCommandHandler;
use crate::command_handler::mcp_server::GolemCliMcpServer;
use crate::log::{Mcp, Output};
use crate::model::WorkerUpdateMode;
use console::strip_ansi_codes;
use golem_templates::model::{GuestLanguage, default_guest_language};
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content, Meta};
use rmcp::{schemars, tool, tool_router, Error as CallToolError, Peer, RoleServer};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;

/// Create new application
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct New {
    /// Application folder name where the new application should be created
    application_name: String,
    /// Languages that the application should support, default langauge : rust
    #[schemars(default="default_guest_language")]
    language: Vec<GuestLanguage>,
}

/// Build all or selected components in the application
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Build {
    component_name: AppOptionalComponentNames,
    build: BuildArgs,
}

/// Deploy all or selected components and HTTP APIs in the application, includes building
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Deploy {
    component_name: AppOptionalComponentNames,
    force_build: ForceBuildArg,
    update_or_redeploy: UpdateOrRedeployArgs,
}

/// Clean all components in the application or by selection
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Clean {
    component_name: AppOptionalComponentNames,
}

/// Try to automatically update all existing workers of the application to the latest version
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdateWorkers {
    component_name: AppOptionalComponentNames,
    /// Update mode - auto or manual, defaults to "auto"
    update_mode: WorkerUpdateMode,
}

/// Redeploy all workers of the application using the latest version
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RedeployWorkers {
    component_name: AppOptionalComponentNames,
}

/// Diagnose possible tooling problems
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Diagnose {
    component_name: AppOptionalComponentNames,
}

/// Run custom command
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CustomCommand{
    command: Vec<String>
}

#[tool_router(router= tool_router_app, vis="pub")]
impl GolemCliMcpServer {
    #[tool(name = "create_new_app", description = "Create a new golem app")]
    pub async fn create_new_app(
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
                tool_name: "create_new_app".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = AppCommandHandler::new(ctx.into());
                match command_new
                    .cmd_new(Some(req.application_name), req.language)
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

    #[tool(name = "custom_command", description = "Custom command in a golem app")]
    pub async fn custom_command(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<CustomCommand>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "custom_command".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = AppCommandHandler::new(ctx.into());
                match command_new.cmd_custom_command(req.command).await {
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

    #[tool(name = "update_app_workers", description = "Update app workers")]
    pub async fn update_app_workers(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<UpdateWorkers>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "update_app_workers".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = AppCommandHandler::new(ctx.into());
                match command_new
                    .cmd_update_workers(req.component_name.component_name, req.update_mode)
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

    #[tool(name = "redeploy_app_workers", description = "Redeploy app workers")]
    pub async fn redeploy_app_workers(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<RedeployWorkers>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "redeploy_app_workers".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = AppCommandHandler::new(ctx.into());
                match command_new
                    .cmd_redeploy_workers(req.component_name.component_name)
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

    #[tool(name = "build_app", description = "Build app")]
    pub async fn build_app(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Build>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "build_app".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = AppCommandHandler::new(ctx.into());
                match tokio::task::spawn_blocking(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        command_new.cmd_build(req.component_name, req.build).await
                    })
                })
                .await
                {
                    Ok(Ok(_)) => Ok(CallToolResult {
                        content: vec![Content::text("Success")],
                        is_error: None,
                    }),
                    Ok(Err(e)) => Ok(CallToolResult {
                        content: vec![Content::text(strip_ansi_codes(&e.to_string()).to_string())],
                        is_error: Some(true),
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

    #[tool(name = "deploy_app", description = "Deploy app")]
    pub async fn deploy_app(
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
                tool_name: "deploy_app".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = AppCommandHandler::new(ctx.into());
                match tokio::task::spawn_blocking(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        command_new
                            .cmd_deploy(req.component_name, req.force_build, req.update_or_redeploy)
                            .await
                    })
                })
                .await
                {
                    Ok(Ok(_)) => Ok(CallToolResult {
                        content: vec![Content::text("Success")],
                        is_error: None,
                    }),
                    Ok(Err(e)) => Ok(CallToolResult {
                        content: vec![Content::text(strip_ansi_codes(&e.to_string()).to_string())],
                        is_error: Some(true),
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

    #[tool(name = "clean_app", description = "Clean app")]
    pub async fn clean_app(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Clean>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "clean_app".to_owned(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = AppCommandHandler::new(ctx.into());

                match command_new.cmd_clean(req.component_name).await {
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

    #[tool(name = "diagnose_app", description = "Diagnose app")]
    pub async fn diagnose_app(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Diagnose>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "diagnose_app".to_string(),
                progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = AppCommandHandler::new(ctx.into());

                match command_new.cmd_diagnose(req.component_name).await {
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
