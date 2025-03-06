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
use crate::config::Config;
use crate::context::{Context, GolemClients};
use crate::error::NonSuccessfulExit;
use crate::fuzzy::{Error, FuzzySearch};
use crate::init_tracing;
use crate::model::app_ext::GolemComponentExtensions;
use crate::model::component::{show_exported_functions, Component};
use crate::model::text::component::{ComponentCreateView, ComponentUpdateView};
use crate::model::text::fmt::{format_export, TextView};
use crate::model::text::help::{
    AvailableComponentNamesHelp, AvailableFunctionNamesHelp, WorkerNameHelp,
};
use crate::model::{ComponentName, WorkerName};
use anyhow::Context as AnyhowContext;
use anyhow::{anyhow, bail};
use colored::Colorize;
use golem_client::api::ComponentClient;
use golem_client::model::DynamicLinkedInstance as DynamicLinkedInstanceOss;
use golem_client::model::DynamicLinkedWasmRpc as DynamicLinkedWasmRpcOss;
use golem_client::model::DynamicLinking as DynamicLinkingOss;
use golem_examples::add_component_by_example;
use golem_examples::model::{ComposableAppGroupName, PackageName};
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
use itertools::Itertools;
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

// CommandHandle is responsible for matching commands and producing CLI output using Context,
// but NOT responsible for storing state (apart from Context itself), those should be part of Context
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
                    Some(build),
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
                    Some(force_build),
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
                    Some(build),
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
                    Some(force_build),
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
                self.ctx.silence_application_context_init();

                let (component_match_kind, component_name, worker_name) =
                    self.match_worker_name(worker_name).await?;

                let component = match self.service_component_by_name(&component_name.0).await? {
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

                        // TODO: we need hashes to reliably detect if "update" deploy is needed
                        //       and for now we should not blindly keep updating, so for now
                        //       only missing one are handled
                        log_action(
                            "Auto deploying",
                            format!(
                                "missing component {}",
                                component_name.0.log_color_highlight()
                            ),
                        );
                        self.deploy(
                            vec![component_name.clone()],
                            None,
                            &ComponentSelectMode::CurrentDir,
                        )
                        .await?;
                        self.service_component_by_name(&component_name.0)
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
                                todo!("NotFound")
                            }
                        }
                    }
                };

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

                match self.ctx.golem_clients().await? {
                    GolemClients::Oss(clients) => match worker_name {
                        Some(worker_name) => {}
                        None => {}
                    },
                    GolemClients::Cloud(_) => {
                        todo!()
                    }
                }

                Ok(())
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
                Ok(())
            }
            GolemCliCommandPartialMatch::AppMissingSubcommandHelp => {
                self.ctx.silence_application_context_init();
                logln("");
                match self
                    .app_ctx_with_selection_mut(vec![], &ComponentSelectMode::All)
                    .await?
                {
                    Some(app_ctx) => app_ctx.log_dynamic_help(&DynamicHelpSections {
                        components: true,
                        custom_commands: true,
                    }),
                    None => {
                        // TODO: maybe add hint that this command should use app manifest
                        Ok(())
                    }
                }
            }
            GolemCliCommandPartialMatch::ComponentMissingSubcommandHelp => {
                self.ctx.silence_application_context_init();
                logln("");
                match self
                    .app_ctx_with_selection_mut(vec![], &ComponentSelectMode::All)
                    .await?
                {
                    Some(app_ctx) => app_ctx.log_dynamic_help(&DynamicHelpSections {
                        components: true,
                        custom_commands: false,
                    }),
                    None => {
                        // TODO: maybe add hint that this command should use app manifest
                        Ok(())
                    }
                }
            }
            GolemCliCommandPartialMatch::WorkerInvokeMissingWorkerName => {
                logln("");
                log_text_view(&WorkerNameHelp);
                logln("");
                // TODO: maybe also show available component names from app?
                Ok(())
            }
            GolemCliCommandPartialMatch::WorkerInvokeMissingFunctionName { worker_name } => {
                self.ctx.silence_application_context_init();
                logln("");
                log_action(
                    "Checking",
                    format!(
                        "provided worker name: {}",
                        worker_name.0.log_color_highlight()
                    ),
                );
                let component_name = {
                    let _indent = Self::nested_text_view_ident();
                    let (component_name_match_kind, component_name, worker_name) =
                        self.match_worker_name(worker_name).await?;
                    logln(format!(
                        "[{}] component name: {} / worker_name: {}, {}",
                        "ok".green(),
                        component_name.0.log_color_highlight(),
                        worker_name
                            .as_ref()
                            .map(|s| s.0.as_str())
                            .unwrap_or("-")
                            .log_color_highlight(),
                        match component_name_match_kind {
                            ComponentNameMatchKind::AppCurrentDir =>
                                "component was selected based on current dir",
                            ComponentNameMatchKind::App =>
                                "component was selected from current application",
                            ComponentNameMatchKind::Unknown => "",
                        }
                    ));
                    component_name
                };
                logln("");
                if let Ok(Some(component)) = self.service_component_by_name(&component_name.0).await
                {
                    log_text_view(&AvailableFunctionNamesHelp {
                        component_name: component_name.0,
                        function_names: show_exported_functions(&component.metadata.exports),
                    });
                    logln("");
                }
                Ok(())
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

    async fn build(
        &mut self,
        component_names: Vec<ComponentName>,
        build: Option<BuildArgs>,
        default_component_select_mode: &ComponentSelectMode,
    ) -> anyhow::Result<()> {
        if let Some(build) = build {
            self.ctx.set_steps_filter(build.step.into_iter().collect());
            self.ctx
                .set_skip_up_to_date_checks(build.force_build.force_build);
        }

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
        force_build: Option<ForceBuildArg>,
        default_component_select_mode: &ComponentSelectMode,
    ) -> anyhow::Result<()> {
        self.build(
            component_names,
            force_build.map(|force_build| BuildArgs {
                step: vec![],
                force_build,
            }),
            default_component_select_mode,
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

        log_action("Deploying", "components");

        for component_name in &selected_component_names {
            let _indent = LogIndent::new();

            let component_id = self.component_id_by_name(component_name.as_str()).await?;
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
    async fn service_component_by_name(
        &self,
        component_name: &str,
    ) -> anyhow::Result<Option<Component>> {
        match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => {
                let mut components = clients
                    .component
                    .get_components(Some(component_name))
                    .await
                    .map_err(map_service_error)?;
                debug!(components = ?components, "service_component_by_name");
                if !components.is_empty() {
                    Ok(Some(Component::from(components.pop().unwrap())))
                } else {
                    Ok(None)
                }
            }
            GolemClients::Cloud(_) => {
                todo!()
            }
        }
    }

    async fn component_id_by_name(&self, component_name: &str) -> anyhow::Result<Option<Uuid>> {
        Ok(self
            .service_component_by_name(component_name)
            .await?
            .map(|c| c.versioned_component_id.component_id))
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

                let app_ctx = self
                    .app_ctx_with_selection_mut(vec![], &ComponentSelectMode::CurrentDir)
                    .await?;
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

                let app_ctx = self
                    .app_ctx_with_selection_mut(vec![], &ComponentSelectMode::All)
                    .await?;
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

    fn log_view<View: TextView + Serialize + DeserializeOwned>(&self, view: &View) {
        // TODO: handle formats
        view.log();
    }

    fn nested_text_view_ident() -> NestedTextViewIndent {
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
fn map_service_error<E: Debug>(error: E) -> anyhow::Error {
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
