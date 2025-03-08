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

use crate::command::app::AppSubcommand;
use crate::command::component::ComponentSubcommand;
use crate::command::shared_args::{BuildArgs, ForceBuildArg};
use crate::command::worker::WorkerSubcommand;
use crate::command::{
    GolemCliCommand, GolemCliCommandParseResult, GolemCliCommandPartialMatch,
    GolemCliFallbackCommand, GolemCliGlobalFlags, GolemCliSubcommand,
};
use crate::command_handler::app::AppCommandHandler;
use crate::command_handler::component::ComponentCommandHandler;
use crate::command_handler::partial_match::PartialMatchHandler;
use crate::command_handler::worker::WorkerCommandHandler;
use crate::config::Config;
use crate::context::{Context, GolemClients};
use crate::error::NonSuccessfulExit;
use crate::fuzzy::{Error, FuzzySearch};
use crate::init_tracing;
use crate::model::app_ext::GolemComponentExtensions;
use crate::model::component::{function_params_types, show_exported_functions, Component};
use crate::model::invoke_result_view::InvokeResultView;
use crate::model::text::component::{ComponentCreateView, ComponentUpdateView};
use crate::model::text::fmt::{format_export, TextView};
use crate::model::text::help::{
    ArgumentError, AvailableComponentNamesHelp, AvailableFunctionNamesHelp,
    ParameterErrorTableView, WorkerNameHelp,
};
use crate::model::{ComponentName, WorkerName};
use anyhow::Context as AnyhowContext;
use anyhow::{anyhow, bail};
use colored::Colorize;
use golem_client::api::{ComponentClient as ComponentClientOss, WorkerClient as WorkerClientOss};
use golem_client::model::DynamicLinking as DynamicLinkingOss;
use golem_client::model::{AnalysedType, DynamicLinkedInstance as DynamicLinkedInstanceOss};
use golem_client::model::{
    DynamicLinkedWasmRpc as DynamicLinkedWasmRpcOss, InvokeParameters as InvokeParametersOss,
};
use golem_examples::add_component_by_example;
use golem_examples::model::{ComposableAppGroupName, PackageName};
use golem_wasm_rpc::json::OptionallyTypeAnnotatedValueJson;
use golem_wasm_rpc::parse_type_annotated_value;
use golem_wasm_rpc::protobuf::type_annotated_value::TypeAnnotatedValue;
use golem_wasm_rpc_stubgen::commands::app::{
    ApplicationContext, ComponentSelectMode, DynamicHelpSections,
};
use golem_wasm_rpc_stubgen::fs;
use golem_wasm_rpc_stubgen::log::Output::Stdout;
use golem_wasm_rpc_stubgen::log::{
    log_action, logln, set_log_output, LogColorize, LogIndent, LogOutput, Output,
};
use golem_wasm_rpc_stubgen::model::app::{ComponentName as AppComponentName, DependencyType};
use indoc::formatdoc;
use itertools::{EitherOrBoth, Itertools};
use log::error;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fmt::Debug;
use std::future::Future;
use std::path::PathBuf;
use std::process::ExitCode;
use tokio::fs::File;
use tracing::{debug, Level};
use tracing_subscriber::fmt::format;
use uuid::Uuid;

mod app;
mod component;
mod partial_match;
mod worker;

// CommandHandle is responsible for matching commands and producing CLI output using Context,
// but NOT responsible for storing state (apart from Context itself), those should be part of Context.
pub struct CommandHandler {
    pub(crate) ctx: Context,
}

impl CommandHandler {
    fn new(global_flags: &GolemCliGlobalFlags) -> Self {
        Self {
            ctx: Context::new(
                global_flags,
                Config::get_active_profile(
                    &global_flags.config_dir(),
                    global_flags.profile.clone(),
                ),
            ),
        }
    }

    // TODO: match and enrich "-h" and "--help"
    pub async fn handle_args<I, T>(args_iterator: I) -> ExitCode
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let result = match GolemCliCommand::try_parse_from_lenient(args_iterator, true) {
            GolemCliCommandParseResult::FullMatch(command) => {
                init_tracing(command.global_flags.verbosity);

                Self::new(&command.global_flags)
                    .handle_command(command)
                    .await
                    .map(|_| ExitCode::SUCCESS)
            }
            GolemCliCommandParseResult::ErrorWithPartialMatch {
                error,
                fallback_command,
                partial_match,
            } => {
                init_tracing(fallback_command.global_flags.verbosity);
                debug!(partial_match = ?partial_match, "Partial match");
                debug_log_parse_error(&error, &fallback_command);
                error.print().unwrap();

                Self::new(&fallback_command.global_flags)
                    .handle_partial_match(partial_match)
                    .await
                    .map(|_| clamp_exit_code(error.exit_code()))
            }
            GolemCliCommandParseResult::Error {
                error,
                fallback_command,
            } => {
                init_tracing(fallback_command.global_flags.verbosity);
                debug_log_parse_error(&error, &fallback_command);
                error.print().unwrap();

                Ok(clamp_exit_code(error.exit_code()))
            }
        };

        result.unwrap_or_else(|error| {
            if error.downcast_ref::<NonSuccessfulExit>().is_none() {
                // TODO: check if this should be display or debug
                logln("");
                log_error(format!("{}", error));
            }
            ExitCode::FAILURE
        })
    }

    async fn handle_command(&mut self, command: GolemCliCommand) -> anyhow::Result<()> {
        match command.subcommand {
            GolemCliSubcommand::App { subcommand } => self.handle_app_subcommand(subcommand).await,
            GolemCliSubcommand::Component { subcommand } => {
                self.handle_component_subcommand(subcommand).await
            }
            GolemCliSubcommand::Worker { subcommand } => {
                self.handle_worker_subcommand(subcommand).await
            }
            GolemCliSubcommand::Api { .. } => {
                todo!()
            }
            GolemCliSubcommand::Plugin { .. } => {
                todo!()
            }
            GolemCliSubcommand::Server { .. } => {
                todo!()
            }
            GolemCliSubcommand::Cloud { .. } => {
                todo!()
            }
            GolemCliSubcommand::Diagnose => {
                todo!()
            }
            GolemCliSubcommand::Completion => {
                todo!()
            }
        }
    }

    pub async fn required_application_context(
        &self,
    ) -> anyhow::Result<&ApplicationContext<GolemComponentExtensions>> {
        self.ctx
            .application_context()
            .await?
            .ok_or_else(no_application_manifest_found_error)
    }

    pub async fn required_application_context_mut(
        &mut self,
    ) -> anyhow::Result<&mut ApplicationContext<GolemComponentExtensions>> {
        self.ctx
            .application_context_mut()
            .await?
            .ok_or_else(no_application_manifest_found_error)
    }

    async fn required_app_ctx_with_selection_mut(
        &mut self,
        component_names: Vec<ComponentName>,
        default: &ComponentSelectMode,
    ) -> anyhow::Result<&mut ApplicationContext<GolemComponentExtensions>> {
        self.app_ctx_with_selection_mut(component_names, default)
            .await?
            .ok_or_else(no_application_manifest_found_error)
    }

    // TODO: forbid matching the same component multiple times
    async fn app_ctx_with_selection_mut(
        &mut self,
        component_names: Vec<ComponentName>,
        default: &ComponentSelectMode,
    ) -> anyhow::Result<Option<&mut ApplicationContext<GolemComponentExtensions>>> {
        let silent_selection = self.ctx.silent_application_context_init();
        let Some(app_ctx) = self.ctx.application_context_mut().await? else {
            return Ok(None);
        };

        if component_names.is_empty() {
            let _log_output = silent_selection.then(|| LogOutput::new(Output::None));
            app_ctx.select_components(default)?
        } else {
            let fuzzy_search =
                FuzzySearch::new(app_ctx.application.component_names().map(|cn| cn.as_str()));

            let (found, not_found) =
                fuzzy_search.find_many(component_names.iter().map(|cn| cn.0.as_str()));

            if !not_found.is_empty() {
                logln("");
                log_error(format!(
                    "The following requested component names were not found:\n{}",
                    not_found
                        .iter()
                        .map(|error| {
                            match error {
                                Error::Ambiguous {
                                    pattern,
                                    highlighted_options,
                                } => {
                                    format!(
                                        "  - {}, did you mean one of {}?",
                                        pattern.as_str().bold(),
                                        highlighted_options.iter().map(|cn| cn.bold()).join(", ")
                                    )
                                }
                                Error::NotFound { pattern } => {
                                    format!("  - {}", pattern.as_str().bold())
                                }
                            }
                        })
                        .join("\n")
                ));
                logln("");
                log_text_view(&AvailableComponentNamesHelp(
                    app_ctx.application.component_names().cloned().collect(),
                ));

                bail!(NonSuccessfulExit);
            }

            let _log_output = silent_selection.then(|| LogOutput::new(Output::None));
            app_ctx.select_components(&ComponentSelectMode::Explicit(
                found.into_iter().map(|m| m.option.into()).collect(),
            ))?
        }
        Ok(Some(app_ctx))
    }

    async fn app_component_dynamic_linking_oss(
        &mut self,
        component_name: &AppComponentName,
    ) -> anyhow::Result<Option<DynamicLinkingOss>> {
        let app_ctx = self.required_application_context_mut().await?;

        let mut mapping = Vec::new();

        let wasm_rpc_deps = app_ctx
            .application
            .component_wasm_rpc_dependencies(component_name)
            .iter()
            .filter(|dep| dep.dep_type == DependencyType::DynamicWasmRpc)
            .cloned()
            .collect::<Vec<_>>();

        for wasm_rpc_dep in wasm_rpc_deps {
            mapping.push(app_ctx.component_stub_interfaces(&wasm_rpc_dep.name)?);
        }

        if mapping.is_empty() {
            Ok(None)
        } else {
            Ok(Some(DynamicLinkingOss {
                dynamic_linking: HashMap::from_iter(mapping.into_iter().map(|stub_interfaces| {
                    (
                        stub_interfaces.stub_interface_name,
                        DynamicLinkedInstanceOss::WasmRpc(DynamicLinkedWasmRpcOss {
                            target_interface_name: HashMap::from_iter(
                                stub_interfaces.exported_interfaces_per_stub_resource,
                            ),
                        }),
                    )
                })),
            }))
        }
    }

    fn log_view<View: TextView + Serialize + DeserializeOwned>(&self, view: &View) {
        // TODO: handle formats
        view.log();
    }

    fn nested_text_view_indent() -> NestedTextViewIndent {
        // TODO: make it format dependent
        NestedTextViewIndent::new()
    }
}

// Unlike CommandHandler::log_view, always use text format regardless of "context", useful for error messages
fn log_text_view<View: TextView>(view: &View) {
    view.log();
}

struct NestedTextViewIndent {
    log_indent: Option<LogIndent>,
}

// TODO: make it format dependent
// TODO: make it not using unicode on NO_COLOR?
impl NestedTextViewIndent {
    fn new() -> Self {
        logln("╔═");
        Self {
            log_indent: Some(LogIndent::prefix("║ ")),
        }
    }
}

impl Drop for NestedTextViewIndent {
    fn drop(&mut self) {
        if let Some(ident) = self.log_indent.take() {
            drop(ident);
            logln("╚═");
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ComponentNameMatchKind {
    AppCurrentDir,
    App,
    Unknown,
}

fn clamp_exit_code(exit_code: i32) -> ExitCode {
    if exit_code < 0 {
        ExitCode::FAILURE
    } else if exit_code > 255 {
        ExitCode::from(255)
    } else {
        ExitCode::from(exit_code as u8)
    }
}

fn debug_log_parse_error(error: &clap::Error, fallback_command: &GolemCliFallbackCommand) {
    debug!(fallback_command = ?fallback_command, "Fallback command");
    debug!(error = ?error, "Clap error");
    if tracing::enabled!(Level::DEBUG) {
        for (kind, value) in error.context() {
            debug!(kind = %kind, value = %value, "Clap error context");
        }
    }
}

fn log_error<S: AsRef<str>>(message: S) {
    logln(format!(
        "{} {}",
        "error:".log_color_error().to_string(),
        message.as_ref()
    ));
}

// TODO: convert to hintable service error ("port" the current GolemError "From" instances)
fn to_service_error<E: Debug>(error: E) -> anyhow::Error {
    anyhow!(format!("Service error: {:#?}", error))
}

fn no_application_manifest_found_error() -> anyhow::Error {
    logln("");
    log_error("No application manifest(s) found!");
    logln(format!(
        "Switch to a directory that contains an application manifest ({}),",
        "golem.yaml".log_color_highlight()
    ));
    logln(format!(
        "or create a new application with the '{}' subcommand!",
        "app new".log_color_highlight(),
    ));
    anyhow!(NonSuccessfulExit)
}
