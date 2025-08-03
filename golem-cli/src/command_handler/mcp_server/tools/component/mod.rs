use crate::command::shared_args::{
    BuildArgs, ComponentOptionalComponentNames, ComponentTemplateName, ForceBuildArg,
    UpdateOrRedeployArgs,
};
use crate::command_handler::mcp_server::GolemCliMcpServer;
use crate::log::{get_mcp_tool_output, Mcp, Output};
use crate::model::app::DependencyType;
use crate::model::{ComponentName, WorkerUpdateMode};
use crate::{command::GolemCliGlobalFlags, command_handler::component::ComponentCommandHandler};
use console::strip_ansi_codes;
use golem_templates::model::PackageName;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content, Meta};
use rmcp::{schemars, tool, tool_router, ErrorData as CallToolError, Peer, RoleServer};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use url::Url;

pub mod plugin;

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateNewComponentTool {
    /// Template to use to create a new component. Accepted format : <LANGUAGE>/<NAME>
    template: ComponentTemplateName,
    /// Package name in the format: (<Namespace>,<Name>), an array
    component_package_name: PackageName,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListComponentTool {
    /// Accepted formats:
    ///   - <COMPONENT>
    ///   - <PROJECT>/<COMPONENT>
    ///   - <ACCOUNT>/<PROJECT>/<COMPONENT>
    component_name: Option<ComponentName>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CleanComponentTool {
    component_name: ComponentOptionalComponentNames,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetComponentTool {
    /// Accepted formats:
    ///   - <COMPONENT>
    ///   - <PROJECT>/<COMPONENT>
    ///   - <ACCOUNT>/<PROJECT>/<COMPONENT>
    component_name: Option<ComponentName>,
    /// Component version in integer(u64)
    version: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdateWorkersTool {
    /// Accepted formats:
    ///   - <COMPONENT>
    ///   - <PROJECT>/<COMPONENT>
    ///   - <ACCOUNT>/<PROJECT>/<COMPONENT>
    component_name: Option<ComponentName>,
    // default Worker update mode is Automatic
    worker_update_mode: WorkerUpdateMode,
    await_update: bool,
}

/// Redeploy all workers of the selected component using the latest version
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RedeployWorkersTool {
    /// Accepted formats:
    ///   - <COMPONENT>
    ///   - <PROJECT>/<COMPONENT>
    ///   - <ACCOUNT>/<PROJECT>/<COMPONENT>
    component_name: Option<ComponentName>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BuildComponentTool {
    component_names: ComponentOptionalComponentNames,
    build_args: BuildArgs,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FilterTemplatesComponentTool {
    filter: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeployComponentTool {
    component_names: ComponentOptionalComponentNames,
    force_build_arg: ForceBuildArg,
    update_or_redeploy_args: UpdateOrRedeployArgs,
}

/// Add or update a component dependency
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AddDependencyTool {
    /// The name of the component to which the dependency should be added
    component_name: Option<ComponentName>,

    /// The name of the component that will be used as the target component
    target_component_name: Option<ComponentName>,

    /// The path to the local component WASM that will be used as the target
    target_component_path: Option<PathBuf>,

    /// The URL to the remote component WASM that will be used as the target
    #[schemars(with = "String")]
    target_component_url: Option<Url>,

    /// The type of the dependency, defaults to wasm-rpc
    dependency_type: Option<DependencyType>,
}

/// Diagnose possible tooling problems
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DiagnoseComponentTool {
    component_name: ComponentOptionalComponentNames,
}

#[tool_router(router= tool_router_component, vis="pub")]
impl GolemCliMcpServer {
    #[tool(
        name = "create_new_component",
        description = "Create a new golem component in a golem app"
    )]
    pub async fn create_new_component(
        &self,
        _meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<CreateNewComponentTool>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "create_new_component".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ComponentCommandHandler::new(ctx.into());
                match command_new
                    .cmd_new(Some(req.template), Some(req.component_package_name))
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

    #[tool(name = "update_workers", description = "Update workers of a golem app")]
    pub async fn update_workers(
        &self,
        _meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<UpdateWorkersTool>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "update_workers".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ComponentCommandHandler::new(ctx.into());
                match command_new
                    .cmd_update_workers(
                        req.component_name,
                        req.worker_update_mode,
                        req.await_update,
                    )
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

    #[tool(
        name = "redeploy_component_workers",
        description = "Redeploy workers of a golem app"
    )]
    pub async fn redeploy_component_workers(
        &self,
        _meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<RedeployWorkersTool>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "redeploy_component_workers".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ComponentCommandHandler::new(ctx.into());
                match command_new.cmd_redeploy_workers(req.component_name).await {
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

    #[tool(
        name = "build_components",
        description = "Build components in a golem app"
    )]
    pub async fn build_components(
        &self,
        _meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<BuildComponentTool>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "build_components".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ComponentCommandHandler::new(ctx.into());
                match tokio::task::spawn_blocking(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        command_new
                            .cmd_build(req.component_names, req.build_args)
                            .await
                    })
                })
                .await
                {
                    Ok(Ok(_)) => Ok(CallToolResult {
                        content: get_mcp_tool_output()
                            .into_iter()
                            .map(Content::text)
                            .collect(),
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

    #[tool(
        name = "filter_templates",
        description = "Filter templates by filter string provided by golem"
    )]
    pub async fn filter_templates(
        &self,
        _meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<FilterTemplatesComponentTool>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "filter_templates".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ComponentCommandHandler::new(ctx.into());

                command_new.cmd_templates(req.filter);
                Ok(CallToolResult {
                    content: get_mcp_tool_output()
                        .into_iter()
                        .map(Content::text)
                        .collect(),
                    is_error: None,
                })
            }
            Err(e) => Ok(CallToolResult {
                content: vec![Content::text(strip_ansi_codes(&e.to_string()).to_string())],
                is_error: Some(true),
            }),
        }
    }

    #[tool(
        name = "deploy_components",
        description = "Deploy components of golem app"
    )]
    pub async fn deploy_components(
        &self,
        _meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<DeployComponentTool>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "deploy_components".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ComponentCommandHandler::new(ctx.into());
                match tokio::task::spawn_blocking(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        command_new
                            .cmd_deploy(
                                req.component_names,
                                req.force_build_arg,
                                req.update_or_redeploy_args,
                            )
                            .await
                    })
                })
                .await
                {
                    Ok(Ok(_)) => Ok(CallToolResult {
                        content: get_mcp_tool_output()
                            .into_iter()
                            .map(Content::text)
                            .collect(),
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

    #[tool(name = "list_components", description = "List components in golem app")]
    pub async fn list_components(
        &self,
        _meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<ListComponentTool>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "list_components".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ComponentCommandHandler::new(ctx.into());

                match command_new.cmd_list(req.component_name).await {
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

    #[tool(
        name = "clean_components",
        description = "Clean components in golem app"
    )]
    pub async fn clean_components(
        &self,
        _meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<CleanComponentTool>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "clean_components".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ComponentCommandHandler::new(ctx.into());

                match command_new.cmd_clean(req.component_name).await {
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

    #[tool(
        name = "get_component",
        description = "get component details in golem app"
    )]
    pub async fn get_components(
        &self,
        _meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<GetComponentTool>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "get_component".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ComponentCommandHandler::new(ctx.into());

                match command_new.cmd_get(req.component_name, req.version).await {
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

    #[tool(
        name = "add_dependency_to_a_component",
        description = "Add dependency to a component in golem app"
    )]
    pub async fn add_dependency_to_a_component(
        &self,
        _meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<AddDependencyTool>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "add_dependency_to_a_component".to_string(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ComponentCommandHandler::new(ctx.into());

                match command_new
                    .cmd_add_dependency(
                        req.component_name,
                        req.target_component_name,
                        req.target_component_path,
                        req.target_component_url,
                        req.dependency_type,
                    )
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

    #[tool(
        name = "diagnose_components",
        description = "Diagnose components in golem app"
    )]
    pub async fn diagnose_components(
        &self,
        _meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<DiagnoseComponentTool>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "diagnose_components".to_string(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = ComponentCommandHandler::new(ctx.into());

                match command_new.cmd_diagnose(req.component_name).await {
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
