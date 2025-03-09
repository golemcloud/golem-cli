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

use crate::command::worker::WorkerSubcommand;
use crate::command_handler::app::AppCommandHandler;
use crate::command_handler::component::ComponentCommandHandler;
use crate::command_handler::log::Log;
use crate::command_handler::CommandHandler;
use crate::context::GolemClients;
use crate::error::{to_service_error, NonSuccessfulExit};
use crate::fuzzy::{Error, FuzzySearch};
use crate::model::component::{function_params_types, show_exported_functions, Component};
use crate::model::invoke_result_view::InvokeResultView;
use crate::model::text::fmt::{format_export, log_error, log_text_view};
use crate::model::text::help::{
    ArgumentError, AvailableComponentNamesHelp, AvailableFunctionNamesHelp,
    ParameterErrorTableView, WorkerNameHelp,
};
use crate::model::{ComponentName, ComponentNameMatchKind, WorkerName};
use anyhow::{anyhow, bail};
use colored::Colorize;
use golem_client::api::WorkerClient as WorkerClientOss;
use golem_client::model::InvokeParameters as InvokeParametersOss;
use golem_wasm_rpc::json::OptionallyTypeAnnotatedValueJson;
use golem_wasm_rpc::parse_type_annotated_value;
use golem_wasm_rpc_stubgen::commands::app::ComponentSelectMode;
use golem_wasm_rpc_stubgen::log::{log_action, logln, LogColorize};
use itertools::{EitherOrBoth, Itertools};

pub trait WorkerCommandHandler {
    fn base(&self) -> &CommandHandler;
    fn base_mut(&mut self) -> &mut CommandHandler;

    async fn handle_worker_subcommand(
        &mut self,
        subcommand: WorkerSubcommand,
    ) -> anyhow::Result<()> {
        match subcommand {
            WorkerSubcommand::Invoke {
                worker_name,
                function_name,
                arguments,
                enqueue,
            } => {
                self.base_mut().ctx.silence_app_context_init();

                let (component_match_kind, component_name, worker_name) =
                    self.match_worker_name(worker_name).await?;

                let component = match self
                    .base()
                    .service_component_by_name(&component_name.0)
                    .await?
                {
                    Some(component) => component,
                    None => {
                        let should_deploy = match component_match_kind {
                            ComponentNameMatchKind::AppCurrentDir => true,
                            ComponentNameMatchKind::App => true,
                            ComponentNameMatchKind::Unknown => false,
                        };

                        if !should_deploy {
                            logln("");
                            log_error(format!(
                                "Component {} not found, and not part of the current application",
                                component_name.0.log_color_highlight()
                            ));
                            // TODO: fuzzy match from service to list components
                            bail!(NonSuccessfulExit)
                        }

                        // TODO: we will need hashes to reliably detect if "update" deploy is needed
                        //       and for now we should not blindly keep updating, so for now
                        //       only missing one are handled
                        log_action(
                            "Auto deploying",
                            format!(
                                "missing component {}",
                                component_name.0.log_color_highlight()
                            ),
                        );
                        self.base_mut()
                            .deploy(
                                vec![component_name.clone()],
                                None,
                                &ComponentSelectMode::CurrentDir,
                            )
                            .await?;
                        self.base()
                            .service_component_by_name(&component_name.0)
                            .await?
                            .ok_or_else(|| {
                                anyhow!("Component ({}) not found after deployment", component_name)
                            })?
                    }
                };

                let component_functions = show_exported_functions(&component.metadata.exports);
                let fuzzy_search = FuzzySearch::new(component_functions.iter().map(|s| s.as_str()));
                let function_name = match fuzzy_search.find(&function_name) {
                    Ok(function_name) => function_name.option,
                    Err(error) => {
                        // TODO: extract common ambiguous messages?
                        match error {
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
                                logln("Did you mean one of".to_string());
                                for option in highlighted_options {
                                    logln(format!(" - {}", option.bold()));
                                }
                                logln("?");
                                logln("");
                                log_text_view(&AvailableFunctionNamesHelp {
                                    component_name: component_name.0,
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
                                    component_name: component_name.0,
                                    function_names: component_functions,
                                });

                                bail!(NonSuccessfulExit);
                            }
                        }
                    }
                };

                if enqueue {
                    log_action(
                        "Enqueueing",
                        format!(
                            "invocation for worker {} / {} / {} ",
                            component_name.0.blue().bold(),
                            worker_name
                                .as_ref()
                                .map(|wn| wn.0.as_str())
                                .unwrap_or("-")
                                .green()
                                .bold(),
                            format_export(&function_name)
                        ),
                    );
                } else {
                    log_action(
                        "Invoking",
                        format!(
                            "worker {} / {} / {} ",
                            component_name.0.blue().bold(),
                            worker_name
                                .as_ref()
                                .map(|wn| wn.0.as_str())
                                .unwrap_or("-")
                                .green()
                                .bold(),
                            format_export(&function_name)
                        ),
                    );
                }

                let arguments = wave_args_to_invoke_args(&component, &function_name, arguments)?;

                let result_view = match self.base().ctx.golem_clients().await? {
                    GolemClients::Oss(clients) => {
                        let result = match worker_name {
                            Some(worker_name) => {
                                if enqueue {
                                    clients
                                        .worker
                                        .invoke_function(
                                            &component.versioned_component_id.component_id,
                                            worker_name.0.as_str(),
                                            None, // TODO: idempotency key
                                            function_name.as_str(),
                                            &InvokeParametersOss { params: arguments },
                                        )
                                        .await
                                        .map_err(to_service_error)?;
                                    None
                                } else {
                                    Some(
                                        clients
                                            .worker
                                            .invoke_and_await_function(
                                                &component.versioned_component_id.component_id,
                                                worker_name.0.as_str(),
                                                None, // TODO: idempotency key
                                                function_name.as_str(),
                                                &InvokeParametersOss { params: arguments },
                                            )
                                            .await
                                            .map_err(to_service_error)?,
                                    )
                                }
                            }
                            None => {
                                if enqueue {
                                    clients
                                        .worker
                                        .invoke_function_without_name(
                                            &component.versioned_component_id.component_id,
                                            None, // TODO: idempotency key
                                            function_name.as_str(),
                                            &InvokeParametersOss { params: arguments },
                                        )
                                        .await
                                        .map_err(to_service_error)?;
                                    None
                                } else {
                                    Some(
                                        clients
                                            .worker
                                            .invoke_and_await_function_without_name(
                                                &component.versioned_component_id.component_id,
                                                None, // TODO: idempotency key
                                                function_name.as_str(),
                                                &InvokeParametersOss { params: arguments },
                                            )
                                            .await
                                            .map_err(to_service_error)?,
                                    )
                                }
                            }
                        };
                        // TODO: handle json format
                        result
                            .map(|result| {
                                InvokeResultView::try_parse_or_json(
                                    result,
                                    &component,
                                    function_name.as_str(),
                                )
                            })
                            .transpose()?
                    }
                    GolemClients::Cloud(_) => {
                        todo!()
                    }
                };

                match result_view {
                    Some(view) => {
                        logln("");
                        self.base().log_view(&view);
                    }
                    None => {
                        log_action("Enqueued", "invocation");
                    }
                }

                Ok(())
            }
        }
    }

    async fn match_worker_name(
        &mut self,
        worker_name: WorkerName,
    ) -> anyhow::Result<(ComponentNameMatchKind, ComponentName, Option<WorkerName>)> {
        fn to_opt_worker_name(worker_name: &str) -> Option<WorkerName> {
            (worker_name != "-").then(|| worker_name.into())
        }

        let segments = worker_name.0.split("/").collect::<Vec<&str>>();
        match segments.len() {
            // <WORKER>
            1 => {
                let worker_name = segments[0];

                self.base_mut()
                    .opt_select_app_components(vec![], &ComponentSelectMode::CurrentDir)?;

                let app_ctx = self.base_mut().ctx.app_context();
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

                        Ok((
                            ComponentNameMatchKind::AppCurrentDir,
                            selected_component_names
                                .iter()
                                .next()
                                .unwrap()
                                .as_str()
                                .into(),
                            to_opt_worker_name(worker_name),
                        ))
                    }
                    None => {
                        logln("");
                        log_error(
                            "Cannot infer the component name for the worker as the current directory is not part of an application."
                        );
                        logln("");
                        logln(
                            "Switch to an application directory or specify the full component name as part of the worker name!",
                        );
                        logln("");
                        log_text_view(&WorkerNameHelp);
                        // TODO: hint for deployed component names?
                        bail!(NonSuccessfulExit);
                    }
                }
            }
            // <COMPONENT>/<WORKER>
            2 => {
                let component_name = segments[0];
                let worker_name = segments[1];

                if worker_name.is_empty() {
                    logln("");
                    log_error("Missing component part in worker name!");
                    logln("");
                    log_text_view(&WorkerNameHelp);
                    logln("");
                    bail!(NonSuccessfulExit);
                }

                if worker_name.is_empty() {
                    logln("");
                    log_error("Missing worker part in worker name!");
                    logln("");
                    log_text_view(&WorkerNameHelp);
                    logln("");
                    bail!(NonSuccessfulExit);
                }

                self.base_mut()
                    .opt_select_app_components(vec![], &ComponentSelectMode::All)?;

                let app_ctx = self.base_mut().ctx.app_context();
                let app_ctx = app_ctx.opt()?;
                match app_ctx {
                    Some(app_ctx) => {
                        let fuzzy_search = FuzzySearch::new(
                            app_ctx.application.component_names().map(|cn| cn.as_str()),
                        );
                        match fuzzy_search.find(component_name) {
                            Ok(match_) => Ok((
                                ComponentNameMatchKind::App,
                                match_.option.into(),
                                to_opt_worker_name(worker_name),
                            )),
                            Err(error) => match error {
                                Error::Ambiguous {
                                    highlighted_options,
                                    ..
                                } => {
                                    logln("");
                                    log_error(format!(
                                        "The requested application component name ({}) is ambiguous.",
                                        component_name.log_color_error_highlight()
                                    ));
                                    logln("");
                                    logln("Did you mean one of".to_string());
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
                                    Ok((
                                        ComponentNameMatchKind::Unknown,
                                        component_name.into(),
                                        to_opt_worker_name(worker_name),
                                    ))
                                }
                            },
                        }
                    }
                    None => Ok((
                        ComponentNameMatchKind::Unknown,
                        component_name.into(),
                        to_opt_worker_name(worker_name),
                    )),
                }
            }
            // <PROJECT>/<COMPONENT>/<WORKER>
            3 => todo!(),
            // <ACCOUNT>/<PROJECT>/<COMPONENT>/<WORKER>
            4 => todo!(),
            _ => {
                logln("");
                log_error(format!(
                    "Cannot parse worker name: {}",
                    worker_name.0.log_color_error_highlight()
                ));
                logln("");
                log_text_view(&WorkerNameHelp);
                bail!(NonSuccessfulExit);
            }
        }
    }
}

fn wave_args_to_invoke_args(
    component: &Component,
    function_name: &str,
    wave_args: Vec<String>,
) -> anyhow::Result<Vec<OptionallyTypeAnnotatedValueJson>> {
    let types = function_params_types(&component, function_name)?;

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
                .into_iter()
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
                .into_iter()
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

    Ok(type_annotated_values
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| anyhow!(err))?
        .into_iter()
        .map(|tav| tav.try_into())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| anyhow!("Failed to convert type annotated value: {err}"))?)
}

impl WorkerCommandHandler for CommandHandler {
    fn base(&self) -> &CommandHandler {
        self
    }

    fn base_mut(&mut self) -> &mut CommandHandler {
        self
    }
}
