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
use crate::command::shared_args::{AppOptionalComponentNames, BuildArgs, ForceBuildArg};
use crate::command::worker::WorkerSubcommand;
use crate::command::{
    GolemCliCommand, GolemCliCommandParseResult, GolemCliCommandPartialMatch,
    GolemCliFallbackCommand, GolemCliGlobalFlags, GolemCliSubcommand,
};
use crate::config::Config;
use crate::context::{Context, GolemClients};
use crate::error::NonSuccessfulExit;
use crate::fuzzy::{Error, FuzzySearch};
use crate::init_tracing;
use crate::model::app_ext::GolemComponentExtensions;
use crate::model::component::{Component, ComponentView};
use crate::model::text::component::{ComponentCreateView, ComponentUpdateView};
use crate::model::text::fmt::TextFormat;
use crate::model::{ComponentName, WorkerName};
use anyhow::Context as AnyhowContext;
use anyhow::{anyhow, bail};
use colored::Colorize;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use golem_client::api::ComponentClient;
use golem_client::model::DynamicLinkedInstance as DynamicLinkedInstanceOss;
use golem_client::model::DynamicLinkedWasmRpc as DynamicLinkedWasmRpcOss;
use golem_client::model::DynamicLinking as DynamicLinkingOss;
use golem_examples::add_component_by_example;
use golem_examples::model::{ComposableAppGroupName, PackageName};
use golem_wasm_ast::analysis::analysed_type::option;
use golem_wasm_rpc_stubgen::commands::app::{
    ApplicationContext, ComponentSelectMode, DynamicHelpSections,
};
use golem_wasm_rpc_stubgen::fs;
use golem_wasm_rpc_stubgen::log::{
    log, log_action, logln, set_log_output, LogColorize, LogIndent, Output,
};
use golem_wasm_rpc_stubgen::model::app::{ComponentName as AppComponentName, DependencyType};
use itertools::Itertools;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ffi::OsString;
use std::fmt::Debug;
use std::path::PathBuf;
use std::process::ExitCode;
use tokio::fs::File;
use tracing::{debug, Level};
use uuid::Uuid;

pub struct CommandHandler {
    ctx: Context,
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
            GolemCliSubcommand::Worker { subcommand } => match subcommand {
                WorkerSubcommand::Invoke {
                    worker_name,
                    function_name,
                    arguments,
                    enqueue,
                } => match self.ctx.golem_clients().await? {
                    GolemClients::Oss(_) => {
                        todo!()
                    }
                    GolemClients::Cloud(_) => {
                        todo!()
                    }
                },
            },
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

    async fn handle_app_subcommand(&mut self, subcommand: AppSubcommand) -> anyhow::Result<()> {
        match subcommand {
            AppSubcommand::New {
                application_name,
                language,
            } => {
                let app_dir = PathBuf::from(&application_name);
                if app_dir.exists() {
                    bail!(
                        "Application directory already exists: {}",
                        app_dir.log_color_error_highlight()
                    );
                }

                // TODO: check for no parent manifests

                fs::create_dir_all(&app_dir)?;
                log_action(
                    "Created",
                    format!(
                        "application directory: {}",
                        app_dir.display().to_string().log_color_highlight()
                    ),
                );

                {
                    let _indent = LogIndent::new();
                    for language in language.language {
                        let Some(language_examples) = self.ctx.templates().get(&language) else {
                            bail!(
                                "No template found for {}, currently supported languages: {}",
                                language.to_string().log_color_error_highlight(),
                                self.ctx.templates().keys().join(", ")
                            );
                        };

                        let default_examples = language_examples
                            .get(&ComposableAppGroupName::default())
                            .expect("No default template found for the selected language");

                        // TODO:
                        assert_eq!(
                            default_examples.components.len(),
                            1,
                            "Expected exactly one default component template"
                        );
                        let (_, default_component_example) =
                            &default_examples.components.iter().next().unwrap();

                        // TODO: better default names
                        let component_package_name = PackageName::from_string(format!(
                            "sample:{}",
                            language.to_string().to_lowercase()
                        ))
                        .unwrap(); // TODO: from args optionally

                        match add_component_by_example(
                            default_examples.common.as_ref(),
                            default_component_example,
                            &app_dir,
                            &component_package_name,
                        ) {
                            Ok(()) => {
                                log_action(
                                    "Added",
                                    format!(
                                        "new app component: {}",
                                        component_package_name
                                            .to_string_with_colon()
                                            .log_color_highlight()
                                    ),
                                );
                            }
                            Err(error) => {
                                bail!("Failed to add new app component: {}", error)
                            }
                        }
                    }
                }

                std::env::set_current_dir(&app_dir)?;
                let Some(app_ctx) = self.ctx.application_context().await? else {
                    return Ok(());
                };

                logln("");
                app_ctx.log_dynamic_help(&DynamicHelpSections {
                    components: true,
                    custom_commands: true,
                })?;

                Ok(())
            }
            AppSubcommand::Build {
                component_name,
                build,
            } => {
                self.build(
                    component_name.component_name,
                    build,
                    &ComponentSelectMode::All,
                )
                .await
            }
            AppSubcommand::Deploy {
                component_name,
                force_build,
            } => {
                self.deploy(
                    component_name.component_name,
                    force_build,
                    &ComponentSelectMode::All,
                )
                .await
            }
            AppSubcommand::Clean { component_name } => {
                self.clean(component_name.component_name, &ComponentSelectMode::All)
                    .await
            }
            AppSubcommand::CustomCommand(command) => {
                if command.len() != 1 {
                    bail!(
                        "Expected exactly one custom subcommand, got: {}",
                        command.join(" ").log_color_error_highlight()
                    );
                }

                self.required_application_context()
                    .await?
                    .custom_command(&command[0])?;

                Ok(())
            }
        }
    }

    async fn handle_component_subcommand(
        &mut self,
        subcommand: ComponentSubcommand,
    ) -> anyhow::Result<()> {
        match subcommand {
            ComponentSubcommand::New { .. } => {
                todo!()
            }
            ComponentSubcommand::Build {
                component_name,
                build,
            } => {
                self.build(
                    component_name.component_name,
                    build,
                    &ComponentSelectMode::CurrentDir,
                )
                .await
            }
            ComponentSubcommand::Deploy {
                component_name,
                force_build,
            } => {
                self.deploy(
                    component_name.component_name,
                    force_build,
                    &ComponentSelectMode::CurrentDir,
                )
                .await
            }
            ComponentSubcommand::Clean { component_name } => {
                self.clean(
                    component_name.component_name,
                    &ComponentSelectMode::CurrentDir,
                )
                .await
            }
        }
    }

    async fn handle_partial_match(
        &mut self,
        partial_match: GolemCliCommandPartialMatch,
    ) -> anyhow::Result<()> {
        match partial_match {
            GolemCliCommandPartialMatch::AppNewMissingLanguage
            | GolemCliCommandPartialMatch::ComponentNewMissingLanguage => {
                logln(format!(
                    "\n{}",
                    "Available languages and templates:".underline().bold(),
                ));
                for (language, templates) in self.ctx.templates() {
                    logln(format!("- {}", language.to_string().bold()));
                    for (group, template) in templates {
                        if group.as_str() != "default" {
                            panic!("TODO: handle non-default groups")
                        }
                        // TODO: strip template names (preferably in golem-examples)
                        for template in template.components.values() {
                            logln(format!(
                                "  - {}: {}",
                                template.name.as_str().bold(),
                                template.description,
                            ));
                        }
                    }
                }
            }
            GolemCliCommandPartialMatch::AppMissingSubcommandHelp => {
                set_log_output(Output::None);
                let Some(app_ctx) = self.ctx.application_context_mut().await? else {
                    // TODO: maybe add hint that this command should use app manifest
                    return Ok(());
                };
                app_ctx.select_components(&ComponentSelectMode::All)?;
                set_log_output(Output::Stderr);
                logln("");
                app_ctx.log_dynamic_help(&DynamicHelpSections {
                    components: true,
                    custom_commands: true,
                })?;
            }
            GolemCliCommandPartialMatch::ComponentMissingSubcommandHelp => {
                // TODO: code dup with AppMissingSubcommandHelp?
                set_log_output(Output::None);
                let Some(app_ctx) = self.ctx.application_context_mut().await? else {
                    // TODO: maybe add hint that this command should use app manifest
                    return Ok(());
                };
                app_ctx.select_components(&ComponentSelectMode::CurrentDir)?;
                set_log_output(Output::Stderr);
                logln("");
                app_ctx.log_dynamic_help(&DynamicHelpSections {
                    components: true,
                    custom_commands: false,
                })?;
            }
            GolemCliCommandPartialMatch::WorkerInvokeMissingWorkerName => {
                /*
                logln(format!("\n{}", "Existing workers:".underline().bold()));
                logln("...");
                logln("To see all workers use.. TODO");
                */
                todo!()
            }
            GolemCliCommandPartialMatch::WorkerInvokeMissingFunctionName { worker_name } => {
                // TODO: search by selected component workers, then by all workers

                logln(format!(
                    "\n{}",
                    format!("Available functions for {}:", worker_name)
                        .underline()
                        .bold(),
                ));
                logln("...");
                todo!()
            }
        }

        Ok(())
    }

    pub async fn required_application_context(
        &self,
    ) -> anyhow::Result<&ApplicationContext<GolemComponentExtensions>> {
        self.ctx
            .application_context()
            .await?
            .ok_or_else(|| no_application_manifest_found())
    }

    pub async fn required_application_context_mut(
        &mut self,
    ) -> anyhow::Result<&mut ApplicationContext<GolemComponentExtensions>> {
        self.ctx
            .application_context_mut()
            .await?
            .ok_or_else(|| no_application_manifest_found())
    }

    async fn required_app_ctx_with_selection_mut(
        &mut self,
        component_names: Vec<ComponentName>,
        default: &ComponentSelectMode,
    ) -> anyhow::Result<&mut ApplicationContext<GolemComponentExtensions>> {
        self.app_ctx_with_selection_mut(component_names, default)
            .await?
            .ok_or_else(|| no_application_manifest_found())
    }

    // TODO: forbid matching the same component multiple times
    async fn app_ctx_with_selection_mut(
        &mut self,
        component_names: Vec<ComponentName>,
        default: &ComponentSelectMode,
    ) -> anyhow::Result<Option<&mut ApplicationContext<GolemComponentExtensions>>> {
        let Some(app_ctx) = self.ctx.application_context_mut().await? else {
            return Ok(None);
        };

        if component_names.is_empty() {
            app_ctx.select_components(default)?
        } else {
            let fuzzy_search =
                FuzzySearch::new(app_ctx.application.component_names().map(|cn| cn.as_str()));

            let (found, not_found) =
                fuzzy_search.find_many(component_names.iter().map(|cn| cn.0.as_str()));

            if !found.is_empty() {
                log_error(format!(
                    "The following requested component names are not found:\n{}",
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
                logln(
                    "Available application components:"
                        .bold()
                        .underline()
                        .to_string(),
                );
                for component_name in app_ctx.application.component_names() {
                    logln(format!("  - {}", component_name));
                }
                logln("");

                bail!(NonSuccessfulExit);
            }

            app_ctx.select_components(&ComponentSelectMode::Explicit(
                found.into_iter().map(|m| m.option.into()).collect(),
            ))?
        }

        Ok(Some(app_ctx))
    }

    async fn build(
        &mut self,
        component_names: Vec<ComponentName>,
        build: BuildArgs,
        default_component_select_mode: &ComponentSelectMode,
    ) -> anyhow::Result<()> {
        self.ctx.set_steps_filter(build.step.into_iter().collect());
        self.ctx
            .set_skip_up_to_date_checks(build.force_build.force_build);

        self.required_app_ctx_with_selection_mut(component_names, default_component_select_mode)
            .await?
            .build()
            .await?;

        Ok(())
    }

    async fn clean(
        &mut self,
        component_names: Vec<ComponentName>,
        default_component_select_mode: &ComponentSelectMode,
    ) -> anyhow::Result<()> {
        self.required_app_ctx_with_selection_mut(component_names, default_component_select_mode)
            .await?
            .clean()?;

        Ok(())
    }

    async fn deploy(
        &mut self,
        component_names: Vec<ComponentName>,
        force_build: ForceBuildArg,
        default_component_select_mode: &ComponentSelectMode,
    ) -> anyhow::Result<()> {
        self.build(
            component_names,
            BuildArgs {
                step: vec![],
                force_build,
            },
            &default_component_select_mode,
        )
        .await?;

        // TODO: hash <-> version check for skipping deploy

        let selected_component_names = self
            .required_application_context()
            .await?
            .selected_component_names()
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        log_action("Updating", "components");

        for component_name in &selected_component_names {
            let _indent = LogIndent::new();

            let component_id = self.component_id_by_name(component_name).await?;
            let app_ctx = self.required_application_context().await?;
            let component_linked_wasm_path = app_ctx
                .application
                .component_linked_wasm(component_name, self.ctx.build_profile());
            let component_linked_wasm = File::open(&component_linked_wasm_path)
                .await
                .with_context(|| {
                    anyhow!(
                        "Failed to open component linked WASM at {}",
                        component_linked_wasm_path
                            .display()
                            .to_string()
                            .log_color_error_highlight()
                    )
                })?;

            let component_properties = &app_ctx
                .application
                .component_properties(component_name, self.ctx.build_profile())
                .clone();
            let component_extensions = &component_properties.extensions;
            let component_dynamic_linking = self
                .app_component_dynamic_linking_oss(component_name)
                .await?;

            match &component_id {
                Some(component_id) => {
                    log_action(
                        "Updating",
                        format!(
                            "component {}",
                            component_name.as_str().log_color_highlight()
                        ),
                    );
                    let _indent = Self::nested_text_view_ident();
                    match self.ctx.golem_clients().await? {
                        GolemClients::Oss(clients) => {
                            let component = clients
                                .component
                                .update_component(
                                    component_id,
                                    Some(&component_extensions.component_type),
                                    component_linked_wasm,
                                    None,         // TODO:
                                    None::<File>, // TODO:
                                    component_dynamic_linking.as_ref(),
                                )
                                .await
                                .map_err(map_service_error)?;
                            self.log_view(&ComponentUpdateView(Component::from(component).into()));
                        }
                        GolemClients::Cloud(_) => {
                            todo!()
                        }
                    }
                }
                None => {
                    log_action(
                        "Creating",
                        format!(
                            "component {}",
                            component_name.as_str().log_color_highlight()
                        ),
                    );
                    let _indent = Self::nested_text_view_ident();
                    match self.ctx.golem_clients().await? {
                        GolemClients::Oss(clients) => {
                            let component = clients
                                .component
                                .create_component(
                                    component_name.as_str(),
                                    Some(&component_extensions.component_type),
                                    component_linked_wasm,
                                    None,         // TODO:
                                    None::<File>, // TODO:
                                    component_dynamic_linking.as_ref(),
                                )
                                .await
                                .map_err(map_service_error)?;
                            self.log_view(&ComponentCreateView(Component::from(component).into()));
                        }
                        GolemClients::Cloud(_) => {
                            todo!()
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // TODO: we might want to have a filter for batch name lookups on the server side
    // TODO: also the search returns all versions
    // TODO: maybe add transient or persistent cache for all the meta
    async fn component_id_by_name(
        &self,
        component_name: &AppComponentName,
    ) -> anyhow::Result<Option<Uuid>> {
        match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => {
                let components = clients
                    .component
                    .get_components(Some(component_name.as_str()))
                    .await
                    .map_err(map_service_error)?;
                debug!(components = ?components, "component_id_by_name");
                if components.len() >= 1 {
                    Ok(Some(components[0].versioned_component_id.component_id))
                } else {
                    Ok(None)
                }
            }
            GolemClients::Cloud(_) => {
                todo!()
            }
        }
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

    async fn parse_worker_name(&mut self, worker_name: &WorkerName) -> anyhow::Result<()> {
        /*let worker_name_segments = worker_name.0.split("/").collect::<Vec<&str>>();
        match worker_name_segments.len() {
            1 => {
                let app_ctx = self
                    .app_ctx_with_selection_mut(vec![], &ComponentSelectMode::CurrentDir)
                    .await?;
                match app_ctx {
                    Some(app_ctx) => {
                        let selected_component_names = app_ctx.selected_component_names();
                        selected_component_names
                    }
                    None => {}
                }
            }
            2 => {}
            3 => todo!(),
            4 => todo!(),
            _ => todo!(),
        }*/

        todo!()
    }

    fn log_view<View: TextFormat + Serialize + DeserializeOwned>(&self, view: &View) {
        // TODO: handle formats
        view.log();
    }

    fn nested_text_view_ident() -> NestedTextViewIndent {
        // TODO: make it format dependent
        NestedTextViewIndent::new()
    }
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
    log("\nerror: ".log_color_error().to_string());
    logln(message.as_ref());
    logln("");
}

// TODO: convert to hintable service error ("port" the current GolemError "From" instances)
fn map_service_error<E: Debug>(error: E) -> anyhow::Error {
    anyhow!(format!("Service error: {:#?}", error))
}

fn no_application_manifest_found() -> anyhow::Error {
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
