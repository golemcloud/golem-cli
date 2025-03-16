// Copyright 2024-2025 Golem Cloud
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::cloud::AccountId;
use crate::command::shared_args::{
    NewWorkerArgument, StreamArgs, WorkerFunctionArgument, WorkerFunctionName, WorkerNameArg,
};
use crate::command::worker::WorkerSubcommand;
use crate::command_handler::Handlers;
use crate::connect_output::ConnectOutput;
use crate::context::{Context, GolemClients};
use crate::error::service::{AnyhowMapServiceError, ServiceError};
use crate::error::NonSuccessfulExit;
use crate::fuzzy::{Error, FuzzySearch};
use crate::model::component::{function_params_types, show_exported_functions, Component};
use crate::model::invoke_result_view::InvokeResultView;
use crate::model::text::fmt::{
    format_export, format_worker_name_match, log_error, log_fuzzy_match, log_text_view, log_warn,
};
use crate::model::text::help::{
    ArgumentError, AvailableComponentNamesHelp, AvailableFunctionNamesHelp, ComponentNameHelp,
    ParameterErrorTableView, WorkerNameHelp,
};
use crate::model::text::worker::{WorkerCreateView, WorkerGetView};
use crate::model::to_oss::ToOss;
use crate::model::{
    ComponentName, ComponentNameMatchKind, Format, IdempotencyKey, ProjectName,
    WorkerConnectOptions, WorkerMetadata, WorkerMetadataView, WorkerName, WorkerNameMatch,
    WorkersMetadataResponseView,
};
use anyhow::{anyhow, bail, Context as AnyhowContext};
use bytes::Bytes;
use colored::control::SHOULD_COLORIZE;
use colored::Colorize;
use futures_util::{future, pin_mut, SinkExt, StreamExt};
use golem_client::api::WorkerClient as WorkerClientOss;
use golem_client::model::{
    InvokeParameters as InvokeParametersOss, ScanCursor,
    WorkerCreationRequest as WorkerCreationRequestOss,
};
use golem_cloud_client::api::WorkerClient as WorkerClientCloud;
use golem_cloud_client::model::{
    InvokeParameters as InvokeParametersCloud, WorkerCreationRequest as WorkerCreationRequestCloud,
};
use golem_common::model::{ComponentType, WorkerEvent};
use golem_wasm_rpc::json::OptionallyTypeAnnotatedValueJson;
use golem_wasm_rpc::parse_type_annotated_value;
use golem_wasm_rpc_stubgen::commands::app::ComponentSelectMode;
use golem_wasm_rpc_stubgen::log::{log_action, logln, LogColorize};
use itertools::{EitherOrBoth, Itertools};
use native_tls::TlsConnector;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::{task, time};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async_tls_with_config, tungstenite, Connector};
use tracing::{debug, error, info, trace};
use url::Url;
use uuid::Uuid;

pub struct WorkerCommandHandler {
    ctx: Arc<Context>,
}

impl WorkerCommandHandler {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    pub async fn handle_command(&mut self, subcommand: WorkerSubcommand) -> anyhow::Result<()> {
        match subcommand {
            WorkerSubcommand::New {
                worker_name,
                arguments,
                env,
            } => self.new_worker(worker_name, arguments, env).await,
            WorkerSubcommand::Invoke {
                worker_name,
                function_name,
                arguments,
                enqueue,
                idempotency_key,
                stream,
                stream_args,
            } => {
                self.invoke(
                    worker_name,
                    &function_name,
                    arguments,
                    enqueue,
                    idempotency_key,
                    stream,
                    stream_args,
                )
                .await
            }
            WorkerSubcommand::Get { worker_name } => self.get(worker_name).await,
            WorkerSubcommand::Delete { worker_name } => self.delete(worker_name).await,
            WorkerSubcommand::List {
                component_name,
                filter: filters,
                scan_cursor,
                max_count,
                precise,
            } => {
                self.list(
                    component_name.component_name,
                    filters,
                    scan_cursor,
                    max_count,
                    precise,
                )
                .await
            }
            WorkerSubcommand::Stream {
                worker_name,
                stream_args,
            } => self.stream(worker_name, stream_args).await,
            WorkerSubcommand::Interrupt { worker_name } => self.interrupt(worker_name).await,
            WorkerSubcommand::Resume { worker_name } => self.resume(worker_name).await,
            WorkerSubcommand::SimulateCrash { worker_name } => {
                self.simulate_crash(worker_name).await
            }
        }
    }

    async fn new_worker(
        &mut self,
        worker_name: WorkerNameArg,
        arguments: Vec<NewWorkerArgument>,
        env: Vec<(String, String)>,
    ) -> anyhow::Result<()> {
        self.ctx.silence_app_context_init().await;

        let worker_name = worker_name.worker_name;
        let mut worker_name_match = self.match_worker_name(worker_name).await?;
        let component = self
            .ctx
            .component_handler()
            .component_by_name_with_auto_deploy(
                worker_name_match.project.as_ref(),
                worker_name_match.component_name_match_kind,
                &worker_name_match.component_name,
            )
            .await?;

        if component.component_type == ComponentType::Ephemeral
            && worker_name_match.worker_name.is_some()
        {
            log_error("Cannot use explicit name for ephemeral worker!");
            logln("");
            logln("Use '-' as worker name for ephemeral workers");
            logln("");
            bail!(NonSuccessfulExit);
        }

        if worker_name_match.worker_name.is_none() {
            worker_name_match.worker_name = Some(Uuid::new_v4().to_string().into());
        }
        let worker_name = worker_name_match.worker_name.clone().unwrap().0;

        log_action(
            "Creating",
            format!(
                "new worker {}",
                format_worker_name_match(&worker_name_match)
            ),
        );

        match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => clients
                .worker
                .launch_new_worker(
                    &component.versioned_component_id.component_id,
                    &WorkerCreationRequestOss {
                        name: worker_name.clone(),
                        args: arguments,
                        env: env.into_iter().collect(),
                    },
                )
                .await
                .map(|_| ())
                .map_service_error()?,
            GolemClients::Cloud(clients) => clients
                .worker
                .launch_new_worker(
                    &component.versioned_component_id.component_id,
                    &WorkerCreationRequestCloud {
                        name: worker_name.clone(),
                        args: arguments,
                        env: env.into_iter().collect(),
                    },
                )
                .await
                .map(|_| ())
                .map_service_error()?,
        };

        logln("");
        self.ctx.log_handler().log_view(&WorkerCreateView {
            component_name: worker_name_match.component_name,
            worker_name: Some(worker_name.into()),
        });

        Ok(())
    }

    async fn invoke(
        &mut self,
        worker_name: WorkerNameArg,
        function_name: &WorkerFunctionName,
        arguments: Vec<WorkerFunctionArgument>,
        enqueue: bool,
        idempotency_key: Option<IdempotencyKey>,
        stream: bool,
        stream_args: StreamArgs,
    ) -> anyhow::Result<()> {
        self.ctx.silence_app_context_init().await;

        fn new_idempotency_key() -> IdempotencyKey {
            let key = IdempotencyKey::new();
            log_action(
                "Using",
                format!("generated idempotency key: {}", key.0.log_color_highlight()),
            );
            key
        }

        let idempotency_key = match idempotency_key {
            Some(idempotency_key) if idempotency_key.0 == "-" => new_idempotency_key(),
            Some(idempotency_key) => {
                log_action(
                    "Using",
                    format!(
                        "requested idempotency key: {}",
                        idempotency_key.0.log_color_highlight()
                    ),
                );
                idempotency_key
            }
            None => new_idempotency_key(),
        };

        let worker_name_match = self.match_worker_name(worker_name.worker_name).await?;

        let component = self
            .ctx
            .component_handler()
            .component_by_name_with_auto_deploy(
                worker_name_match.project.as_ref(),
                worker_name_match.component_name_match_kind,
                &worker_name_match.component_name,
            )
            .await?;

        let component_functions = show_exported_functions(&component.metadata.exports);
        let fuzzy_search = FuzzySearch::new(component_functions.iter().map(|s| s.as_str()));
        let function_name = match fuzzy_search.find(function_name) {
            Ok(match_) => {
                log_fuzzy_match(&match_);
                match_.option
            }
            Err(error) => match error {
                Error::Ambiguous {
                    highlighted_options,
                    ..
                } => {
                    logln("");
                    log_error(format!(
                        "The requested function name ({}) is ambiguous.",
                        function_name.log_color_error_highlight()
                    ));
                    logln("");
                    logln("Did you mean one of");
                    for option in highlighted_options {
                        logln(format!(" - {}", option.bold()));
                    }
                    logln("?");
                    logln("");
                    log_text_view(&AvailableFunctionNamesHelp {
                        component_name: worker_name_match.component_name.0,
                        function_names: component_functions,
                    });

                    bail!(NonSuccessfulExit);
                }
                Error::NotFound { .. } => {
                    logln("");
                    log_error(format!(
                        "The requested function name ({}) was not found.",
                        function_name.log_color_error_highlight()
                    ));
                    logln("");
                    log_text_view(&AvailableFunctionNamesHelp {
                        component_name: worker_name_match.component_name.0,
                        function_names: component_functions,
                    });

                    bail!(NonSuccessfulExit);
                }
            },
        };

        if enqueue {
            log_action(
                "Enqueueing",
                format!(
                    "invocation for worker {} / {}",
                    format_worker_name_match(&worker_name_match),
                    format_export(&function_name)
                ),
            );
        } else {
            log_action(
                "Invoking",
                format!(
                    "worker {} / {} ",
                    format_worker_name_match(&worker_name_match),
                    format_export(&function_name)
                ),
            );
        }

        let arguments = wave_args_to_invoke_args(&component, &function_name, arguments)?;

        let connect_handle = match worker_name_match.worker_name.clone() {
            Some(worker_name) => {
                if stream {
                    let connection = connect_to_worker(
                        self.ctx.worker_service_url().clone(),
                        self.ctx.auth_token().await?,
                        component.versioned_component_id.component_id.clone(),
                        worker_name.0,
                        stream_args.into(),
                        self.ctx.allow_insecure(),
                        self.ctx.format(),
                    )
                    .await?;
                    Some(tokio::task::spawn(async move {
                        connection.read_messages().await
                    }))
                } else {
                    None
                }
            }
            None => None,
        };

        let result = match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => match &worker_name_match.worker_name {
                Some(worker_name) => {
                    if enqueue {
                        clients
                            .worker
                            .invoke_function(
                                &component.versioned_component_id.component_id,
                                &worker_name.0,
                                Some(&idempotency_key.0),
                                function_name.as_str(),
                                &InvokeParametersOss { params: arguments },
                            )
                            .await
                            .map_service_error()?;
                        None
                    } else {
                        Some(
                            clients
                                .worker
                                .invoke_and_await_function(
                                    &component.versioned_component_id.component_id,
                                    &worker_name.0,
                                    Some(&idempotency_key.0),
                                    function_name.as_str(),
                                    &InvokeParametersOss { params: arguments },
                                )
                                .await
                                .map_service_error()?,
                        )
                    }
                }
                None => {
                    if enqueue {
                        clients
                            .worker
                            .invoke_function_without_name(
                                &component.versioned_component_id.component_id,
                                Some(&idempotency_key.0),
                                function_name.as_str(),
                                &InvokeParametersOss { params: arguments },
                            )
                            .await
                            .map_service_error()?;
                        None
                    } else {
                        Some(
                            clients
                                .worker
                                .invoke_and_await_function_without_name(
                                    &component.versioned_component_id.component_id,
                                    Some(&idempotency_key.0),
                                    function_name.as_str(),
                                    &InvokeParametersOss { params: arguments },
                                )
                                .await
                                .map_service_error()?,
                        )
                    }
                }
            },
            GolemClients::Cloud(clients) => match worker_name_match.worker_name {
                Some(worker_name) => {
                    if enqueue {
                        clients
                            .worker
                            .invoke_function(
                                &component.versioned_component_id.component_id,
                                &worker_name.0,
                                Some(&idempotency_key.0),
                                function_name.as_str(),
                                &InvokeParametersCloud { params: arguments },
                            )
                            .await
                            .map_service_error()?;
                        None
                    } else {
                        Some(
                            clients
                                .worker
                                .invoke_and_await_function(
                                    &component.versioned_component_id.component_id,
                                    &worker_name.0,
                                    Some(&idempotency_key.0),
                                    function_name.as_str(),
                                    &InvokeParametersCloud { params: arguments },
                                )
                                .await
                                .map_service_error()?,
                        )
                    }
                }
                None => {
                    if enqueue {
                        clients
                            .worker
                            .invoke_function_without_name(
                                &component.versioned_component_id.component_id,
                                Some(&idempotency_key.0),
                                function_name.as_str(),
                                &InvokeParametersCloud { params: arguments },
                            )
                            .await
                            .map_service_error()?;
                        None
                    } else {
                        Some(
                            clients
                                .worker
                                .invoke_and_await_function_without_name(
                                    &component.versioned_component_id.component_id,
                                    Some(&idempotency_key.0),
                                    function_name.as_str(),
                                    &InvokeParametersCloud { params: arguments },
                                )
                                .await
                                .map_service_error()?,
                        )
                    }
                }
            }
            .to_oss(),
        };

        connect_handle.iter().for_each(|handle| handle.abort());

        match result {
            Some(result) => {
                logln("");
                self.ctx
                    .log_handler()
                    .log_view(&InvokeResultView::new_invoke(
                        idempotency_key,
                        result,
                        &component,
                        function_name.as_str(),
                    ));
            }
            None => {
                log_action("Enqueued", "invocation");
                self.ctx
                    .log_handler()
                    .log_view(&InvokeResultView::new_enqueue(idempotency_key));
            }
        };

        Ok(())
    }

    async fn get(&mut self, worker_name: WorkerNameArg) -> anyhow::Result<()> {
        self.ctx.silence_app_context_init().await;
        let worker_name_match = self.match_worker_name(worker_name.worker_name).await?;
        let (component, worker_name) = self
            .component_by_worker_name_match(&worker_name_match)
            .await?;

        let result = match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => {
                let result = clients
                    .worker
                    .get_worker_metadata(
                        &component.versioned_component_id.component_id,
                        &worker_name.0,
                    )
                    .await
                    .map_service_error()?;

                WorkerMetadata::from_oss(worker_name_match.component_name, result)
            }
            GolemClients::Cloud(clients) => {
                let result = clients
                    .worker
                    .get_worker_metadata(
                        &component.versioned_component_id.component_id,
                        &worker_name.0,
                    )
                    .await
                    .map_service_error()?;

                WorkerMetadata::from_cloud(worker_name_match.component_name, result)
            }
        };

        self.ctx
            .log_handler()
            .log_view(&WorkerGetView::from(result));

        Ok(())
    }

    async fn delete(&mut self, worker_name: WorkerNameArg) -> anyhow::Result<()> {
        self.ctx.silence_app_context_init().await;
        let worker_name_match = self.match_worker_name(worker_name.worker_name).await?;
        let (component, worker_name) = self
            .component_by_worker_name_match(&worker_name_match)
            .await?;

        log_action(
            "Deleting",
            format!("worker {}", format_worker_name_match(&worker_name_match)),
        );

        let _ = match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => {
                clients
                    .worker
                    .delete_worker(
                        &component.versioned_component_id.component_id,
                        &worker_name.0,
                    )
                    .await
                    .map(|_| ())
                    .map_service_error()?;
            }
            GolemClients::Cloud(clients) => clients
                .worker
                .get_worker_metadata(
                    &component.versioned_component_id.component_id,
                    &worker_name.0,
                )
                .await
                .map(|_| ())
                .map_service_error()?,
        };

        log_action(
            "Deleted",
            format!("worker {}", format_worker_name_match(&worker_name_match)),
        );

        Ok(())
    }

    async fn stream(
        &mut self,
        worker_name: WorkerNameArg,
        stream_args: StreamArgs,
    ) -> anyhow::Result<()> {
        self.ctx.silence_app_context_init().await;
        let worker_name_match = self.match_worker_name(worker_name.worker_name).await?;
        let (component, worker_name) = self
            .component_by_worker_name_match(&worker_name_match)
            .await?;

        log_action(
            "Connecting",
            format!("to worker {}", format_worker_name_match(&worker_name_match)),
        );

        let connection = connect_to_worker(
            self.ctx.worker_service_url().clone(),
            self.ctx.auth_token().await?,
            component.versioned_component_id.component_id.clone(),
            worker_name.0.clone(),
            stream_args.into(),
            self.ctx.allow_insecure(),
            self.ctx.format(),
        )
        .await?;

        connection.read_messages().await;

        Ok(())
    }

    async fn interrupt(&mut self, worker_name: WorkerNameArg) -> anyhow::Result<()> {
        self.ctx.silence_app_context_init().await;
        let worker_name_match = self.match_worker_name(worker_name.worker_name).await?;
        let (component, worker_name) = self
            .component_by_worker_name_match(&worker_name_match)
            .await?;

        log_action(
            "Interrupting",
            format!("worker {}", format_worker_name_match(&worker_name_match)),
        );

        let _ = self
            .interrupt_worker(&component, &worker_name, false)
            .await?;

        log_action(
            "Interrupted",
            format!("worker {}", format_worker_name_match(&worker_name_match)),
        );

        Ok(())
    }

    async fn resume(&mut self, worker_name: WorkerNameArg) -> anyhow::Result<()> {
        self.ctx.silence_app_context_init().await;
        let worker_name_match = self.match_worker_name(worker_name.worker_name).await?;
        let (component, worker_name) = self
            .component_by_worker_name_match(&worker_name_match)
            .await?;

        log_action(
            "Resuming",
            format!("worker {}", format_worker_name_match(&worker_name_match)),
        );

        let _ = self
            .interrupt_worker(&component, &worker_name, true)
            .await?;

        log_action(
            "Resumed",
            format!("worker {}", format_worker_name_match(&worker_name_match)),
        );

        Ok(())
    }

    async fn simulate_crash(&mut self, worker_name: WorkerNameArg) -> anyhow::Result<()> {
        self.ctx.silence_app_context_init().await;
        let worker_name_match = self.match_worker_name(worker_name.worker_name).await?;
        let (component, worker_name) = self
            .component_by_worker_name_match(&worker_name_match)
            .await?;

        log_action(
            "Simulating crash",
            format!(
                "for worker {}",
                format_worker_name_match(&worker_name_match)
            ),
        );

        let _ = self
            .interrupt_worker(&component, &worker_name, true)
            .await?;

        log_action(
            "Simulated crash",
            format!(
                "for worker {}",
                format_worker_name_match(&worker_name_match)
            ),
        );

        Ok(())
    }

    async fn list(
        &self,
        component_name: Option<ComponentName>,
        filters: Vec<String>,
        scan_cursor: Option<ScanCursor>,
        max_count: Option<u64>,
        precise: bool,
    ) -> anyhow::Result<()> {
        let selected_components = self
            .ctx
            .component_handler()
            .must_select_components_by_app_or_name(component_name.as_ref())
            .await?;

        if scan_cursor.is_some() && selected_components.component_names.len() != 1 {
            log_error(format!(
                "Cursor cannot be used with multiple components selected! ({})",
                selected_components
                    .component_names
                    .iter()
                    .map(|cn| cn.0.log_color_highlight())
                    .join(", ")
            ));
            logln("");
            logln("Switch to an application directory with only one component or explicitly specify the requested component name.");
            logln("");
            bail!(NonSuccessfulExit);
        }

        let scan_cursor = scan_cursor.as_ref().map(scan_cursor_to_string);

        let mut view = WorkersMetadataResponseView::default();

        for component_name in &selected_components.component_names {
            match self
                .ctx
                .component_handler()
                .component_by_name(selected_components.project.as_ref(), component_name)
                .await?
            {
                Some(component) => {
                    let mut current_scan_cursor = scan_cursor.clone();
                    loop {
                        let result_cursor = match self.ctx.golem_clients().await? {
                            GolemClients::Oss(clients) => {
                                let results = clients
                                    .worker
                                    .get_workers_metadata(
                                        &component.versioned_component_id.component_id,
                                        Some(&filters),
                                        current_scan_cursor.as_deref(),
                                        max_count.or(Some(self.ctx.http_batch_size())),
                                        Some(precise),
                                    )
                                    .await
                                    .map_service_error()?;

                                view.workers.extend(results.workers.into_iter().map(|meta| {
                                    WorkerMetadataView::from(WorkerMetadata::from_oss(
                                        component_name.clone(),
                                        meta,
                                    ))
                                }));

                                results.cursor
                            }
                            GolemClients::Cloud(clients) => {
                                let results = clients
                                    .worker
                                    .get_workers_metadata(
                                        &component.versioned_component_id.component_id,
                                        Some(&filters),
                                        current_scan_cursor.as_deref(),
                                        max_count.or(Some(self.ctx.http_batch_size())),
                                        Some(precise),
                                    )
                                    .await
                                    .map_service_error()?;

                                view.workers.extend(results.workers.into_iter().map(|meta| {
                                    WorkerMetadataView::from(WorkerMetadata::from_cloud(
                                        component_name.clone(),
                                        meta,
                                    ))
                                }));

                                results.cursor.to_oss()
                            }
                        };

                        match result_cursor {
                            Some(next_cursor) => {
                                if max_count.is_none() {
                                    current_scan_cursor = Some(scan_cursor_to_string(&next_cursor));
                                } else {
                                    view.cursors.insert(
                                        component_name.to_string().clone(),
                                        scan_cursor_to_string(&next_cursor),
                                    );
                                    break;
                                }
                            }
                            None => {
                                break;
                            }
                        }
                    }
                }
                None => {
                    log_warn(format!(
                        "Component not found: {}",
                        component_name.0.log_color_error_highlight()
                    ));
                }
            }
        }

        self.ctx.log_handler().log_view(&view);

        Ok(())
    }

    pub async fn match_worker_name(
        &mut self,
        worker_name: WorkerName,
    ) -> anyhow::Result<WorkerNameMatch> {
        fn to_opt_worker_name(worker_name: String) -> Option<WorkerName> {
            (worker_name != "-").then(|| worker_name.into())
        }

        let segments = worker_name.0.split("/").collect::<Vec<&str>>();
        match segments.len() {
            // <WORKER>
            1 => {
                let worker_name = segments[0].to_string();

                self.ctx
                    .app_handler()
                    .opt_select_components(vec![], &ComponentSelectMode::CurrentDir)
                    .await?;

                let app_ctx = self.ctx.app_context_lock().await;
                let app_ctx = app_ctx.opt()?;
                match app_ctx {
                    Some(app_ctx) => {
                        let selected_component_names = app_ctx.selected_component_names();

                        if selected_component_names.len() != 1 {
                            logln("");
                            log_error(
                                format!("Multiple components were selected based on the current directory: {}",
                                        selected_component_names.iter().map(|cn| cn.as_str().log_color_highlight()).join(", ")),
                            );
                            logln("");
                            logln(
                                "Switch to a different directory with only one component or specify the full or partial component name as part of the worker name!",
                            );
                            logln("");
                            log_text_view(&WorkerNameHelp);
                            logln("");
                            log_text_view(&AvailableComponentNamesHelp(
                                app_ctx.application.component_names().cloned().collect(),
                            ));
                            bail!(NonSuccessfulExit);
                        }

                        Ok(WorkerNameMatch {
                            account_id: None,
                            project: None,
                            component_name_match_kind: ComponentNameMatchKind::AppCurrentDir,
                            component_name: selected_component_names
                                .iter()
                                .next()
                                .unwrap()
                                .as_str()
                                .into(),
                            worker_name: to_opt_worker_name(worker_name),
                        })
                    }
                    None => {
                        logln("");
                        log_error("Cannot infer the component name for the worker as the current directory is not part of an application.");
                        logln("");
                        logln("Switch to an application directory or specify the full component name as part of the worker name!");
                        logln("");
                        log_text_view(&WorkerNameHelp);
                        // TODO: hint for deployed component names?
                        bail!(NonSuccessfulExit);
                    }
                }
            }
            // [ACCOUNT]/[PROJECT]/<COMPONENT>/<WORKER>
            2..=4 => {
                fn empty_checked<'a>(name: &'a str, value: &'a str) -> anyhow::Result<&'a str> {
                    if value.is_empty() {
                        log_error(format!("Missing {} part in worker name!", name));
                        logln("");
                        log_text_view(&ComponentNameHelp);
                        bail!(NonSuccessfulExit);
                    }
                    Ok(value)
                }

                fn empty_checked_account(value: &str) -> anyhow::Result<&str> {
                    empty_checked("account", value)
                }

                fn empty_checked_project(value: &str) -> anyhow::Result<&str> {
                    empty_checked("project", value)
                }

                fn empty_checked_component(value: &str) -> anyhow::Result<&str> {
                    empty_checked("component", value)
                }

                fn empty_checked_worker(value: &str) -> anyhow::Result<&str> {
                    empty_checked("worker", value)
                }

                let (account_id, project_name, component_name, worker_name): (
                    Option<AccountId>,
                    Option<ProjectName>,
                    ComponentName,
                    String,
                ) = match segments.len() {
                    2 => (
                        None,
                        None,
                        empty_checked_component(segments[0])?.into(),
                        empty_checked_component(segments[1])?.into(),
                    ),
                    3 => (
                        None,
                        Some(empty_checked_project(segments[0])?.into()),
                        empty_checked_component(segments[1])?.into(),
                        empty_checked_component(segments[2])?.into(),
                    ),
                    4 => (
                        Some(empty_checked_account(segments[0])?.into()),
                        Some(empty_checked_project(segments[1])?.into()),
                        empty_checked_component(segments[2])?.into(),
                        empty_checked_worker(segments[3])?.into(),
                    ),
                    other => panic!("Unexpected segment count: {}", other),
                };

                if worker_name.is_empty() {
                    logln("");
                    log_error("Missing component part in worker name!");
                    logln("");
                    log_text_view(&WorkerNameHelp);
                    bail!(NonSuccessfulExit);
                }

                let project = self
                    .ctx
                    .cloud_project_handler()
                    .opt_select_project(account_id.as_ref(), project_name.as_ref())
                    .await?;

                self.ctx
                    .app_handler()
                    .opt_select_components(vec![], &ComponentSelectMode::All)
                    .await?;

                let app_ctx = self.ctx.app_context_lock().await;
                let app_ctx = app_ctx.opt()?;
                match app_ctx {
                    Some(app_ctx) => {
                        let fuzzy_search = FuzzySearch::new(
                            app_ctx.application.component_names().map(|cn| cn.as_str()),
                        );
                        match fuzzy_search.find(&component_name.0) {
                            Ok(match_) => {
                                log_fuzzy_match(&match_);
                                Ok(WorkerNameMatch {
                                    account_id,
                                    project,
                                    component_name_match_kind: ComponentNameMatchKind::App,
                                    component_name: match_.option.into(),
                                    worker_name: to_opt_worker_name(worker_name),
                                })
                            }
                            Err(error) => match error {
                                Error::Ambiguous {
                                    highlighted_options,
                                    ..
                                } => {
                                    logln("");
                                    log_error(format!(
                                        "The requested application component name ({}) is ambiguous.",
                                        component_name.0.log_color_error_highlight()
                                    ));
                                    logln("");
                                    logln("Did you mean one of");
                                    for option in highlighted_options {
                                        logln(format!(" - {}", option.bold()));
                                    }
                                    logln("?");
                                    logln("");
                                    log_text_view(&WorkerNameHelp);
                                    logln("");
                                    log_text_view(&AvailableComponentNamesHelp(
                                        app_ctx.application.component_names().cloned().collect(),
                                    ));

                                    bail!(NonSuccessfulExit);
                                }
                                Error::NotFound { .. } => {
                                    // Assuming non-app component
                                    Ok(WorkerNameMatch {
                                        account_id,
                                        project,
                                        component_name_match_kind: ComponentNameMatchKind::Unknown,
                                        component_name,
                                        worker_name: to_opt_worker_name(worker_name),
                                    })
                                }
                            },
                        }
                    }
                    None => Ok(WorkerNameMatch {
                        account_id,
                        project,
                        component_name_match_kind: ComponentNameMatchKind::Unknown,
                        component_name,
                        worker_name: to_opt_worker_name(worker_name),
                    }),
                }
            }
            _ => {
                logln("");
                log_error(format!(
                    "Failed to parse worker name: {}",
                    worker_name.0.log_color_error_highlight()
                ));
                logln("");
                log_text_view(&WorkerNameHelp);
                bail!(NonSuccessfulExit);
            }
        }
    }

    async fn component_by_worker_name_match(
        &mut self,
        worker_name_match: &WorkerNameMatch,
    ) -> anyhow::Result<(Component, WorkerName)> {
        let Some(worker_name) = &worker_name_match.worker_name else {
            log_error("Worker name is required");
            logln("");
            log_text_view(&WorkerNameHelp);
            logln("");
            bail!(NonSuccessfulExit);
        };

        let component = self
            .ctx
            .component_handler()
            .component_by_name(
                worker_name_match.project.as_ref(),
                &worker_name_match.component_name,
            )
            .await?;

        let Some(component) = component else {
            log_error(format!(
                "Component {} not found",
                worker_name_match
                    .component_name
                    .0
                    .log_color_error_highlight()
            ));
            logln("");
            bail!(NonSuccessfulExit);
        };

        Ok((component, worker_name.clone()))
    }

    async fn interrupt_worker(
        &mut self,
        component: &Component,
        worker_name: &WorkerName,
        recover_immediately: bool,
    ) -> anyhow::Result<()> {
        match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => {
                clients
                    .worker
                    .interrupt_worker(
                        &component.versioned_component_id.component_id,
                        &worker_name.0,
                        Some(recover_immediately),
                    )
                    .await
                    .map(|_| ())
                    .map_service_error()?;
            }
            GolemClients::Cloud(clients) => clients
                .worker
                .interrupt_worker(
                    &component.versioned_component_id.component_id,
                    &worker_name.0,
                    Some(recover_immediately),
                )
                .await
                .map(|_| ())
                .map_service_error()?,
        }
        Ok(())
    }
}

fn wave_args_to_invoke_args(
    component: &Component,
    function_name: &str,
    wave_args: Vec<String>,
) -> anyhow::Result<Vec<OptionallyTypeAnnotatedValueJson>> {
    let types = function_params_types(component, function_name)?;

    if types.len() != wave_args.len() {
        logln("");
        log_error(format!(
            "Wrong number of parameters: expected {}, got {}",
            types.len(),
            wave_args.len()
        ));
        logln("");
        log_text_view(&ParameterErrorTableView(
            types
                .into_iter()
                .zip_longest(wave_args)
                .map(|zipped| match zipped {
                    EitherOrBoth::Both(typ, value) => ArgumentError {
                        type_: Some(typ.clone()),
                        value: Some(value),
                        error: None,
                    },
                    EitherOrBoth::Left(typ) => ArgumentError {
                        type_: Some(typ.clone()),
                        value: None,
                        error: Some("missing argument".log_color_error().to_string()),
                    },
                    EitherOrBoth::Right(value) => ArgumentError {
                        type_: None,
                        value: Some(value),
                        error: Some("extra argument".log_color_error().to_string()),
                    },
                })
                .collect::<Vec<_>>(),
        ));
        logln("");
        bail!(NonSuccessfulExit);
    }

    let type_annotated_values = wave_args
        .iter()
        .zip(types.iter())
        .map(|(wave, typ)| parse_type_annotated_value(typ, wave))
        .collect::<Vec<_>>();

    if type_annotated_values
        .iter()
        .any(|parse_result| parse_result.is_err())
    {
        logln("");
        log_error("Argument WAVE parse error(s)!");
        logln("");
        log_text_view(&ParameterErrorTableView(
            type_annotated_values
                .into_iter()
                .zip(types)
                .zip(wave_args)
                .map(|((parsed, typ), value)| (parsed, typ, value))
                .map(|(parsed, typ, value)| ArgumentError {
                    type_: Some(typ.clone()),
                    value: Some(value),
                    error: parsed
                        .err()
                        .map(|err| err.log_color_error_highlight().to_string()),
                })
                .collect::<Vec<_>>(),
        ));
        logln("");
        bail!(NonSuccessfulExit);
    }

    type_annotated_values
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| anyhow!(err))?
        .into_iter()
        .map(|tav| tav.try_into())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| anyhow!("Failed to convert type annotated value: {err}"))
}

fn scan_cursor_to_string(cursor: &ScanCursor) -> String {
    format!("{}/{}", cursor.layer, cursor.cursor)
}

fn parse_worker_error(status: u16, body: Vec<u8>) -> ServiceError {
    let error: anyhow::Result<
        Option<golem_client::Error<golem_client::api::WorkerError>>,
        serde_json::Error,
    > = match status {
        400 => serde_json::from_slice(&body).map(|body| {
            Some(golem_client::Error::Item(
                golem_client::api::WorkerError::Error400(body),
            ))
        }),
        401 => serde_json::from_slice(&body).map(|body| {
            Some(golem_client::Error::Item(
                golem_client::api::WorkerError::Error401(body),
            ))
        }),
        403 => serde_json::from_slice(&body).map(|body| {
            Some(golem_client::Error::Item(
                golem_client::api::WorkerError::Error403(body),
            ))
        }),
        404 => serde_json::from_slice(&body).map(|body| {
            Some(golem_client::Error::Item(
                golem_client::api::WorkerError::Error404(body),
            ))
        }),
        409 => serde_json::from_slice(&body).map(|body| {
            Some(golem_client::Error::Item(
                golem_client::api::WorkerError::Error409(body),
            ))
        }),
        500 => serde_json::from_slice(&body).map(|body| {
            Some(golem_client::Error::Item(
                golem_client::api::WorkerError::Error500(body),
            ))
        }),
        _ => Ok(None),
    };

    match error.ok().flatten() {
        Some(error) => error.into(),
        None => {
            golem_client::Error::<golem_client::api::WorkerError>::unexpected(status, body.into())
                .into()
        }
    }
}

struct WorkerConnection {
    pings: JoinHandle<anyhow::Error>,
    read_messages: JoinHandle<()>,
}

impl WorkerConnection {
    async fn read_messages(self) {
        let pings = self.pings;
        let read_res = self.read_messages;
        pin_mut!(pings, read_res);
        future::select(pings, read_res).await;
    }
}

async fn connect_to_worker(
    worker_service_url: Url,
    auth_token: Option<String>,
    component_id: Uuid,
    worker_name: String,
    connect_options: WorkerConnectOptions,
    allow_insecure: bool,
    format: Format,
) -> anyhow::Result<WorkerConnection> {
    let mut url = worker_service_url;

    let ws_schema = if url.scheme() == "http" { "ws" } else { "wss" };

    url.set_scheme(ws_schema)
        .map_err(|()| anyhow!("Failed to set ws url schema".to_string()))?;
    url.path_segments_mut()
        .map_err(|()| anyhow!("Failed to get url path for ws url".to_string()))?
        .push("v1")
        .push("components")
        .push(&component_id.to_string())
        .push("workers")
        .push(&worker_name)
        .push("connect");

    debug!(url = url.as_str(), "Worker connect");

    let mut request = url
        .to_string()
        .into_client_request()
        .context("Failed to create request")?;

    if let Some(token) = auth_token {
        let headers = request.headers_mut();
        headers.insert("Authorization", format!("Bearer {}", token).parse()?);
    }

    let connector = if allow_insecure {
        Some(Connector::NativeTls(
            TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .build()?,
        ))
    } else {
        None
    };

    let (ws_stream, _) = connect_async_tls_with_config(request, None, false, connector)
        .await
        .map_err(|e| match e {
            tungstenite::error::Error::Http(http_error_response) => {
                let status = http_error_response.status().as_u16();
                match http_error_response.body().clone() {
                    Some(body) => anyhow!(parse_worker_error(status, body)),
                    None => anyhow!("Websocket connect failed, HTTP error: {}", status),
                }
            }
            _ => anyhow!("Websocket connect failed, error: {}", e),
        })?;

    let (mut write, read) = ws_stream.split();

    let pings = task::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(1)); // TODO configure
        let mut cnt: i64 = 1;

        loop {
            interval.tick().await;

            let ping_result = write
                .send(Message::Ping(Bytes::from(cnt.to_ne_bytes().to_vec())))
                .await
                .context("Failed to send ping");

            if let Err(err) = ping_result {
                error!("{}", err);
                break err;
            }

            cnt += 1;
        }
    });

    let output = ConnectOutput::new(connect_options, format);

    let read_messages = task::spawn(async move {
        read.for_each(move |message_or_error| {
            let output = output.clone();
            async move {
                match message_or_error {
                    Err(error) => {
                        error!("Error reading message: {}", error);
                    }
                    Ok(message) => {
                        let instance_connect_msg = match message {
                            Message::Text(str) => {
                                let parsed: serde_json::Result<WorkerEvent> =
                                    serde_json::from_str(str.as_str());

                                match parsed {
                                    Ok(parsed) => Some(parsed),
                                    Err(err) => {
                                        error!("Failed to parse worker connect message: {err}");
                                        None
                                    }
                                }
                            }
                            Message::Binary(data) => {
                                let parsed: serde_json::Result<WorkerEvent> =
                                    serde_json::from_slice(data.as_ref());
                                match parsed {
                                    Ok(parsed) => Some(parsed),
                                    Err(err) => {
                                        error!("Failed to parse worker connect message: {err}");
                                        None
                                    }
                                }
                            }
                            Message::Ping(_) => {
                                trace!("Ignore ping");
                                None
                            }
                            Message::Pong(_) => {
                                trace!("Ignore pong");
                                None
                            }
                            Message::Close(details) => {
                                match details {
                                    Some(closed_frame) => {
                                        info!("Connection Closed: {}", closed_frame);
                                    }
                                    None => {
                                        info!("Connection Closed");
                                    }
                                }
                                None
                            }
                            Message::Frame(f) => {
                                debug!("Ignored unexpected frame {f:?}");
                                None
                            }
                        };

                        match instance_connect_msg {
                            None => {}
                            Some(msg) => match msg {
                                WorkerEvent::StdOut { timestamp, bytes } => {
                                    output
                                        .emit_stdout(
                                            timestamp,
                                            String::from_utf8_lossy(&bytes).to_string(),
                                        )
                                        .await;
                                }
                                WorkerEvent::StdErr { timestamp, bytes } => {
                                    output
                                        .emit_stderr(
                                            timestamp,
                                            String::from_utf8_lossy(&bytes).to_string(),
                                        )
                                        .await;
                                }
                                WorkerEvent::Log {
                                    timestamp,
                                    level,
                                    context,
                                    message,
                                } => {
                                    output.emit_log(timestamp, level, context, message);
                                }
                                WorkerEvent::Close => {} // TODO:
                                WorkerEvent::InvocationStart { .. } => {} // TODO:
                                WorkerEvent::InvocationFinished { .. } => {} // TODO:
                            },
                        }
                    }
                }
            }
        })
        .await;
    });

    Ok(WorkerConnection {
        pings,
        read_messages,
    })
}
