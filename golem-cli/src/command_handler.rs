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
use crate::command::worker::WorkerSubcommand;
use crate::command::{
    GolemCliCommand, GolemCliCommandParseResult, GolemCliCommandPartialMatch,
    GolemCliFallbackCommand, GolemCliGlobalFlags, GolemCliSubcommand,
};
use crate::config::Config;
use crate::context::{Context, GolemClients};
use crate::error::{HandledError, HintError};
use crate::fuzzy::{FuzzyMatchResult, FuzzySearch};
use crate::init_tracing;
use crate::model::app_ext::GolemComponentExtensions;
use crate::model::ComponentName;
use anyhow::bail;
use colored::Colorize;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use golem_examples::add_component_by_example;
use golem_examples::model::{ComposableAppGroupName, PackageName};
use golem_wasm_rpc_stubgen::commands::app::{
    ApplicationContext, ComponentSelectMode, DynamicHelpSections,
};
use golem_wasm_rpc_stubgen::fs;
use golem_wasm_rpc_stubgen::log::{
    log, log_action, logln, set_log_output, LogColorize, LogIndent, Output,
};
use golem_wasm_rpc_stubgen::model::app::ComponentName as AppComponentName;
use itertools::Itertools;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;
use tracing::{debug, Level};

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
                    .handle_command_with_hints(command)
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
                    .handle_partial_match_with_hints(partial_match)
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
            if error.downcast_ref::<HandledError>().is_none() {
                // TODO: check if this should be display or debug
                log_error(format!("{}", error));
            }
            ExitCode::FAILURE
        })
    }

    async fn handle_command_with_hints(&mut self, command: GolemCliCommand) -> anyhow::Result<()> {
        let result = self.handle_command(command).await;
        self.try_match_hint_errors(result).await
    }

    async fn handle_command(&mut self, command: GolemCliCommand) -> anyhow::Result<()> {
        match command.subcommand {
            GolemCliSubcommand::App { subcommand } => self.handle_app_subcommand(subcommand).await,
            GolemCliSubcommand::Component { subcommand } => {
                self.handle_component_subcommand(subcommand).await
            }
            GolemCliSubcommand::Worker { subcommand } => match subcommand {
                WorkerSubcommand::Invoke { .. } => match self.ctx.golem_clients().await? {
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
                step,
                force_build,
            } => {
                self.ctx.set_steps_filter(step.into_iter().collect());
                self.ctx.set_skip_up_to_date_checks(force_build.force_build);

                self.app_ctx_with_selected_app_components_prefer_all(component_name.component_name)
                    .await?
                    .build()
                    .await?;

                Ok(())
            }
            AppSubcommand::Deploy {
                component_name,
                force_build,
            } => {
                self.ctx.set_skip_up_to_date_checks(force_build.force_build);

                let _app_ctx = self
                    .app_ctx_with_selected_app_components_prefer_all(component_name.component_name)
                    .await?;

                todo!()
            }
            AppSubcommand::Clean { component_name } => {
                self.app_ctx_with_selected_app_components_prefer_all(component_name.component_name)
                    .await?
                    .clean()?;

                Ok(())
            }
            AppSubcommand::CustomCommand(command) => {
                if command.len() != 1 {
                    bail!(
                        "Expected exactly one custom subcommand, got: {}",
                        command.join(" ").log_color_error_highlight()
                    );
                }

                self.ctx
                    .required_application_context()
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
            ComponentSubcommand::Build { .. } => {
                todo!()
            }
            ComponentSubcommand::Deploy { .. } => {
                todo!()
            }
            ComponentSubcommand::Clean { .. } => {
                todo!()
            }
        }
    }

    async fn handle_partial_match_with_hints(
        &mut self,
        partial_match: GolemCliCommandPartialMatch,
    ) -> anyhow::Result<()> {
        let result = self.handle_partial_match(partial_match).await;
        self.try_match_hint_errors(result).await
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
                logln(format!("\n{}", "Existing workers:".underline().bold()));
                logln("...");
                logln("To see all workers use.. TODO");
                todo!()
            }
            GolemCliCommandPartialMatch::WorkerInvokeMissingFunctionName { worker_name } => {
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

    async fn try_match_hint_errors(&self, result: anyhow::Result<()>) -> anyhow::Result<()> {
        match result {
            Ok(value) => Ok(value),
            Err(error) => match error.downcast_ref::<HintError>() {
                Some(error) => match self.handle_hint_error(error).await {
                    Ok(()) => Err(HandledError)?,
                    Err(error) => Err(error),
                },
                None => Err(error),
            },
        }
    }

    async fn handle_hint_error(&self, error: &HintError) -> anyhow::Result<()> {
        match error {
            HintError::NoApplicationManifestsFound => {
                log_error("No application manifest(s) found!");
                logln(format!(
                    "Switch to a directory that contains an application manifest ({}),",
                    "golem.yaml".log_color_highlight()
                ));
                logln(format!(
                    "or create a new application with the '{}' subcommand!",
                    "app new".log_color_highlight(),
                ));
                Ok(())
            }
            HintError::ComponentNotFound(_) => {
                todo!()
            }
            HintError::WorkerNotFound(worker_name) => {
                logln(format!(
                    "Dynamic help for worker not found: {}",
                    worker_name
                ));
                Ok(())
            }
        }
    }

    async fn app_ctx_with_selected_app_components_prefer_all(
        &mut self,
        component_names: Vec<ComponentName>,
    ) -> anyhow::Result<&mut ApplicationContext<GolemComponentExtensions>> {
        self.app_ctx_with_selected_app_components(component_names, &ComponentSelectMode::All)
            .await
    }

    async fn app_ctx_with_selected_app_components_prefer_current_dir(
        &mut self,
        component_names: Vec<ComponentName>,
    ) -> anyhow::Result<&mut ApplicationContext<GolemComponentExtensions>> {
        self.app_ctx_with_selected_app_components(component_names, &ComponentSelectMode::CurrentDir)
            .await
    }

    // TODO: forbid matching the same component multiple times
    async fn app_ctx_with_selected_app_components(
        &mut self,
        component_names: Vec<ComponentName>,
        default: &ComponentSelectMode,
    ) -> anyhow::Result<&mut ApplicationContext<GolemComponentExtensions>> {
        let app_ctx = self.ctx.required_application_context_mut().await?;

        if component_names.is_empty() {
            app_ctx.select_components(default)?
        } else {
            let fuzzy_search =
                FuzzySearch::new(app_ctx.application.component_names().map(|cn| cn.as_str()));

            let mut component_names_found =
                Vec::<AppComponentName>::with_capacity(component_names.len());
            let mut component_names_not_found = Vec::<(String, Vec<String>)>::new();

            for component_name in component_names {
                match fuzzy_search.find(component_name.0.as_str()) {
                    FuzzyMatchResult::Found { option, .. } => {
                        component_names_found.push(option.into())
                    }
                    FuzzyMatchResult::Ambiguous {
                        highlighted_options,
                    } => component_names_not_found
                        .push((component_name.0.into(), highlighted_options)),
                    FuzzyMatchResult::NotFound => {
                        component_names_not_found.push((component_name.0.into(), vec![]))
                    }
                }
            }

            if !component_names_not_found.is_empty() {
                log_error(format!(
                    "The following requested component names are not found:\n{}",
                    component_names_not_found
                        .iter()
                        .map(|(component_name, similar_matches)| {
                            if similar_matches.is_empty() {
                                format!("  - {}", component_name.as_str().bold())
                            } else {
                                format!(
                                    "  - {}, did you mean one of {}?",
                                    component_name.as_str().bold(),
                                    similar_matches.iter().map(|cn| cn.bold()).join(", ")
                                )
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

                bail!(HandledError);
            }

            app_ctx.select_components(&ComponentSelectMode::Explicit(component_names_found))?
        }

        Ok(app_ctx)
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
