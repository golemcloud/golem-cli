use crate::command::shared_args::{
    ComponentOptionalComponentName, NewWorkerArgument, StreamArgs, WorkerFunctionArgument,
    WorkerFunctionName, WorkerNameArg,
};
use crate::command::GolemCliGlobalFlags;
use crate::command_handler::mcp_server::GolemCliMcpServer;
use crate::command_handler::worker::WorkerCommandHandler;
use crate::log::{get_mcp_tool_output, Mcp, Output};
use crate::model::{IdempotencyKey, WorkerUpdateMode};
use console::strip_ansi_codes;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content, Meta};
use rmcp::{schemars, tool, tool_router, Error as CallToolError, Peer, RoleServer};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;

/// Create new worker
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct New {
    worker_name: WorkerNameArg,
    /// Worker arguments
    arguments: Vec<NewWorkerArgument>,
    /// Worker environment variables
    env: Vec<(String, String)>,
}

// TODO: json args
/// Invoke (or enqueue invocation for) worker
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Invoke {
    worker_name: WorkerNameArg,
    /// Worker function name to invoke
    function_name: WorkerFunctionName,
    /// Worker function arguments in WAVE format
    arguments: Vec<WorkerFunctionArgument>,
    /// Enqueue invocation, and do not wait for it
    enqueue: bool,
    /// Set idempotency key for the call, use "-" for auto generated key
    idempotency_key: Option<IdempotencyKey>,
    /// Connect to the worker before invoke (the worker must already exist)
    /// and live stream its standard output, error and log channels
    stream: bool,
    stream_args: StreamArgs,
}

/// Get worker metadata
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Get {
    worker_name: WorkerNameArg,
}

/// Deletes a worker
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Delete {
    worker_name: WorkerNameArg,
}

/// List worker metadata
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct List {
    component_name: ComponentOptionalComponentName,
    /// Filter for worker metadata in form of `property op value`.
    ///
    /// Filter examples: `name = worker-name`, `version >= 0`, `status = Running`, `env.var1 = value`.
    /// Can be used multiple times (AND condition is applied between them)
    filter: Vec<String>,

    /// Cursor position, if not provided, starts from the beginning.
    ///
    /// Cursor can be used to get the next page of results, use the cursor returned
    /// in the previous response.
    /// The cursor has the format 'layer/position' where both layer and position are numbers.
    // scan_cursor: Option<ScanCursor>,
    // TODO:// need changes in golem repo, scan_cursor: Option<ScanCursor>,

    /// The maximum the number of returned workers, returns all values is not specified.
    /// When multiple component is selected, then the limit it is applied separately
    max_count: Option<u64>,
    /// When set to true it queries for most up-to-date status for each worker, default is false
    precise: bool,
}

/// Connect to a worker and live stream its standard output, error and log channels
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Stream {
    worker_name: WorkerNameArg,
    stream_args: StreamArgs,
}

/// Updates a worker
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Update {
    worker_name: WorkerNameArg,
    /// Update mode - auto or manual (default is auto)
    mode: Option<WorkerUpdateMode>,
    /// The new version of the updated worker (default is the latest version)
    target_version: Option<u64>,
}

/// Interrupts a running worker
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Interrupt {
    worker_name: WorkerNameArg,
}

/// Resume an interrupted worker
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Resume {
    worker_name: WorkerNameArg,
}

/// Simulates a crash on a worker for testing purposes.
///
/// The worker starts recovering and resuming immediately.
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SimulateCrash {
    worker_name: WorkerNameArg,
}

/// Queries and dumps a worker's full oplog
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Oplog {
    worker_name: WorkerNameArg,
    /// Index of the first oplog entry to get. If missing, the whole oplog is returned
    from: Option<u64>,
    /// Lucene query to look for oplog entries. If missing, the whole oplog is returned
    query: Option<String>,
}

/// Reverts a worker by undoing its last recorded operations
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Revert {
    worker_name: WorkerNameArg,
    /// Revert by oplog index
    last_oplog_index: Option<u64>,
    /// Revert by number of invocations
    number_of_invocations: Option<u64>,
}

/// Cancels an enqueued invocation if it has not started yet
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CancelInvocation {
    worker_name: WorkerNameArg,
    /// Idempotency key of the invocation to be cancelled
    idempotency_key: IdempotencyKey,
}

#[tool_router(router= tool_router_worker, vis="pub")]
impl GolemCliMcpServer {
    #[tool(name = "create_new_worker", description = "Create new worker")]
    pub async fn create_new_worker(
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
                tool_name: "create_new_worker".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = WorkerCommandHandler::new(ctx.into());
                match tokio::task::spawn_blocking(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        command_new
                            .cmd_new(req.worker_name, req.arguments, req.env)
                            .await
                    })
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

    #[tool(name = "list_worker_metadata", description = "List worker metadata")]
    pub async fn list_worker_metadata(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<List>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "list_worker_metadata".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now, can we provide a seperate tool to start the golem server
        )
        .await
        {
            Ok(ctx) => {
                let command_new = WorkerCommandHandler::new(ctx.into());
                match command_new
                    .cmd_list(
                        req.component_name.component_name,
                        req.filter,
                        None, // issue with this, need update in golem repo
                        req.max_count,
                        req.precise,
                    )
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

    #[tool(name = "update_worker", description = "Updates a worker")]
    pub async fn update_worker(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Update>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "update_worker".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = WorkerCommandHandler::new(ctx.into());
                match command_new
                    .cmd_update(
                        req.worker_name,
                        req.mode.unwrap_or(WorkerUpdateMode::Automatic),
                        req.target_version,
                    )
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

    #[tool(name = "get_worker_metadata", description = "Get worker metadata")]
    pub async fn get_worker_metadata(
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
                tool_name: "get_worker_metadata".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = WorkerCommandHandler::new(ctx.into());
                match command_new.cmd_get(req.worker_name).await {
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

    #[tool(name = "delete_worker", description = "Deletes a worker")]
    pub async fn delete_worker(
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
                tool_name: "delete_worker".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = WorkerCommandHandler::new(ctx.into());
                match command_new.cmd_delete(req.worker_name).await {
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

    #[tool(
        name = "invoke_worker",
        description = "Invoke (or enqueue invocation for) worker"
    )]
    pub async fn invoke_worker(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Invoke>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "invoke_worker".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = WorkerCommandHandler::new(ctx.into());
                match tokio::task::spawn_blocking(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        command_new
                            .cmd_invoke(
                                req.worker_name,
                                &req.function_name,
                                req.arguments,
                                req.enqueue,
                                req.idempotency_key,
                                req.stream,
                                req.stream_args,
                            )
                            .await
                    })
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

    #[tool(name = "resume_worker", description = "Resume an interrupted worker")]
    pub async fn resume_worker(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Resume>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "resume_worker".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = WorkerCommandHandler::new(ctx.into());
                match command_new.cmd_resume(req.worker_name).await {
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

    #[tool(name = "interupt_worker", description = "Interrupts a running worker")]
    pub async fn interupt_worker(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Interrupt>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "interupt_worker".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = WorkerCommandHandler::new(ctx.into());
                match command_new.cmd_interrupt(req.worker_name).await {
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

    #[tool(
        name = "simulate_worker_crash",
        description = "Simulates a crash on a worker for testing purposes. The worker starts recovering and resuming immediately."
    )]
    pub async fn simulate_worker_crash(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<SimulateCrash>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "simulate_worker_crash".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = WorkerCommandHandler::new(ctx.into());
                match command_new.cmd_simulate_crash(req.worker_name).await {
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

    #[tool(
        name = "query_worker_oplog",
        description = "Queries and dumps a worker's full oplog"
    )]
    pub async fn query_worker_oplog(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Oplog>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "query_worker_oplog".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = WorkerCommandHandler::new(ctx.into());
                match command_new
                    .cmd_oplog(req.worker_name, req.from, req.query)
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

    #[tool(
        name = "stream_worker_outputs",
        description = "Connect to a worker and live stream its standard output, error and log channels"
    )]
    pub async fn stream_worker_outputs(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Stream>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "stream_worker_outputs".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = WorkerCommandHandler::new(ctx.into());
                match command_new
                    .cmd_stream(req.worker_name, req.stream_args)
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

    #[tool(
        name = "revert_worker_operations",
        description = "Reverts a worker by undoing its last recorded operations"
    )]
    pub async fn revert_worker_operations(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<Revert>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "revert_worker_operations".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = WorkerCommandHandler::new(ctx.into());
                match command_new
                    .cmd_revert(
                        req.worker_name,
                        req.last_oplog_index,
                        req.number_of_invocations,
                    )
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

    #[tool(
        name = "cancel_worker_invocation",
        description = "Cancels an enqueued invocation if it has not started yet"
    )]
    pub async fn cancel_worker_invocation(
        &self,
        meta: Meta,
        client: Peer<RoleServer>,
        Parameters(req): Parameters<CancelInvocation>,
    ) -> Result<CallToolResult, CallToolError> {
        let start_local_server_yes = Arc::new(tokio::sync::RwLock::new(false));

        match crate::context::Context::new(
            GolemCliGlobalFlags::default(),
            Some(Output::Mcp(Mcp {
                client,
                tool_name: "cancel_worker_invocation".to_owned(),
                // progress_token: meta.get_progress_token().ok_or(CallToolError::invalid_params("Progress Token is required to use this tool", None))?
            })),
            start_local_server_yes,
            Box::new(|| Box::pin(async { Ok(()) })), // dummy, not starting anything for now
        )
        .await
        {
            Ok(ctx) => {
                let command_new = WorkerCommandHandler::new(ctx.into());
                match command_new
                    .cmd_cancel_invocation(req.worker_name, req.idempotency_key)
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
