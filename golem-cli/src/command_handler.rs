use crate::command::app::AppSubcommand;
use crate::command::component::ComponentSubcommand;
use crate::command::shared_args::AppOptionalComponentNames;
use crate::command::worker::WorkerSubcommand;
use crate::command::{
    GolemCliCommand, GolemCliCommandParseResult, GolemCliCommandPartialMatch,
    GolemCliFallbackCommand, GolemCliGlobalFlags, GolemCliSubcommand,
};
use crate::config::Config;
use crate::context::{Context, GolemClients};
use crate::error::{HintError, HintedError};
use crate::init_tracing;
use anyhow::bail;
use colored::Colorize;
use golem_examples::add_component_by_example;
use golem_examples::model::{ComposableAppGroupName, PackageName};
use golem_wasm_rpc_stubgen::commands::app::ComponentSelectMode;
use golem_wasm_rpc_stubgen::fs;
use golem_wasm_rpc_stubgen::log::{
    log_action, logln, set_log_output, LogColorize, LogIndent, Output,
};
use itertools::Itertools;
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
                log_parse_error(&error, &fallback_command);
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
                log_parse_error(&error, &fallback_command);
                error.print().unwrap();

                Ok(clamp_exit_code(error.exit_code()))
            }
        };

        result.unwrap_or_else(|error| {
            // TODO: formatting of "stacktraces"
            eprintln!("\n{} {}", "error: ".log_color_error(), error);
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
                let app_ctx = self.ctx.application_context().await?;
                logln("");
                app_ctx.log_dynamic_help()?;

                Ok(())
            }
            AppSubcommand::Build {
                component_name,
                step,
                force_build,
            } => {
                self.ctx
                    .set_component_select_mode(app_component_select_mode(component_name));
                self.ctx.set_steps_filter(step.into_iter().collect());
                self.ctx.set_skip_up_to_date_checks(force_build.force_build);

                let app_ctx = self.ctx.application_context_mut().await?;

                app_ctx.build().await?;

                Ok(())
            }
            AppSubcommand::Deploy {
                component_name,
                force_build,
            } => {
                self.ctx
                    .set_component_select_mode(app_component_select_mode(component_name));
                self.ctx.set_skip_up_to_date_checks(force_build.force_build);

                let _app_ctx = self.ctx.application_context_mut().await?;

                todo!()
            }
            AppSubcommand::Clean { component_name } => {
                self.ctx
                    .set_component_select_mode(app_component_select_mode(component_name));

                let app_ctx = self.ctx.application_context_mut().await?;

                app_ctx.clean()?;

                Ok(())
            }
            AppSubcommand::CustomCommand(command) => {
                if command.len() != 1 {
                    bail!(
                        "Expected exactly one custom subcommand, got: {}",
                        command.join(" ").log_color_error_highlight()
                    );
                }

                let ctx = self.ctx.application_context().await?;
                ctx.custom_command(&command[0])?;

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
        &self,
        partial_match: GolemCliCommandPartialMatch,
    ) -> anyhow::Result<()> {
        self.try_match_hint_errors(self.handle_partial_match(partial_match).await)
            .await
    }

    async fn handle_partial_match(
        &self,
        partial_match: GolemCliCommandPartialMatch,
    ) -> anyhow::Result<()> {
        match partial_match {
            GolemCliCommandPartialMatch::AppNewMissingLanguage
            | GolemCliCommandPartialMatch::ComponentNewMissingLanguage => {
                eprintln!(
                    "{}",
                    "\nAvailable languages and templates:".underline().bold()
                );
                for (language, templates) in self.ctx.templates() {
                    eprintln!("- {}", language.to_string().bold());
                    for (group, template) in templates {
                        if group.as_str() != "default" {
                            panic!("TODO: handle non-default groups")
                        }
                        // TODO: strip template names (preferably in golem-examples)
                        for template in template.components.values() {
                            eprintln!(
                                "  - {}: {}",
                                template.name.as_str().bold(),
                                template.description
                            );
                        }
                    }
                }
            }
            GolemCliCommandPartialMatch::AppMissingSubcommandHelp => {
                set_log_output(Output::None);
                let Ok(app_ctx) = self.ctx.application_context().await else {
                    return Ok(());
                };

                set_log_output(Output::Stderr);
                logln("");
                app_ctx.log_dynamic_help()?;
            }
            GolemCliCommandPartialMatch::WorkerInvokeMissingWorkerName => {
                eprintln!("{}", "\nExisting workers:".underline().bold());
                eprintln!("...");
                eprintln!("To see all workers use.. TODO");
                todo!()
            }
            GolemCliCommandPartialMatch::WorkerInvokeMissingFunctionName { worker_name } => {
                eprintln!(
                    "\n{}",
                    format!("Available functions for {}:", worker_name)
                        .underline()
                        .bold()
                );
                eprintln!("...");
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
                    Ok(()) => Err(HintedError)?,
                    Err(error) => Err(error),
                },
                None => Err(error),
            },
        }
    }

    async fn handle_hint_error(&self, error: &HintError) -> anyhow::Result<()> {
        match error {
            HintError::ComponentNotFound(_) => {
                todo!()
            }
            HintError::WorkerNotFound(worker_name) => {
                eprintln!("Dynamic help for worker not found: {}", worker_name);
                Ok(())
            }
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

fn log_parse_error(error: &clap::Error, fallback_command: &GolemCliFallbackCommand) {
    debug!(fallback_command = ?fallback_command, "Fallback command");
    debug!(error = ?error, "Clap error");
    if tracing::enabled!(Level::DEBUG) {
        for (kind, value) in error.context() {
            debug!(kind = %kind, value = %value, "Clap error context");
        }
    }
}

// TODO:
/*
fn component_select_mode(component_names: AppOptionalComponentNames) -> ComponentSelectMode {
    ComponentSelectMode::current_dir_or_explicit(
        component_names
            .component_name
            .into_iter()
            .map(|cn| cn.0.into())
            .collect(),
    )
}
*/

fn app_component_select_mode(component_names: AppOptionalComponentNames) -> ComponentSelectMode {
    ComponentSelectMode::all_or_explicit(
        component_names
            .component_name
            .into_iter()
            .map(|cn| cn.0.into())
            .collect(),
    )
}
