use crate::command::worker::WorkerSubcommand;
use crate::command::{
    GolemCliCommand, GolemCliCommandParseResult, GolemCliCommandPartialMatch, GolemCliGlobalFlags,
    GolemCliSubcommand,
};
use crate::config::Config;
use crate::context::{Context, GolemClients};
use crate::error::{HintError, HintedError};
use crate::init_tracing;
use crate::model::{ComponentName, WorkerName};
use colored::Colorize;
use golem_client::api::WorkerClient as WorkerClientOss;
use golem_cloud_client::api::WorkerClient as WorkerClientCloud;
use golem_examples::model::GuestLanguage;
use golem_wasm_rpc_stubgen::commands::app::ApplicationSourceMode::Explicit;
use std::ffi::OsString;
use std::future::Future;
use std::process::{exit, ExitCode};
use std::sync::Arc;
use strum::IntoEnumIterator;
use tracing::{debug, Level};
use uuid::Uuid;

pub struct CommandHandler {
    ctx: Context,
}

impl CommandHandler {
    fn new(global_flags: &GolemCliGlobalFlags) -> Self {
        Self {
            ctx: Context::new(Config::get_active_profile(
                &global_flags.config_dir(),
                global_flags.profile.clone(),
            )),
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
                global_flags,
                partial_match,
            } => {
                init_tracing(global_flags.verbosity);

                error.print().unwrap();

                Self::new(&global_flags)
                    .handle_partial_match_with_hints(partial_match)
                    .await
                    .map(|_| clamp_exit_code(error.exit_code()))
            }
            GolemCliCommandParseResult::Error {
                error,
                global_flags,
            } => {
                init_tracing(global_flags.verbosity);

                if tracing::enabled!(Level::DEBUG) {
                    for (kind, value) in error.context() {
                        debug!(kind = %kind, value = %value, "Error context");
                    }
                }

                error.print().unwrap();

                Ok(clamp_exit_code(error.exit_code()))
            }
        };

        result.unwrap_or_else(|err| {
            // TODO: formatting / matching
            eprintln!("{}", err);
            ExitCode::FAILURE
        })
    }

    async fn handle_command_with_hints(&self, command: GolemCliCommand) -> anyhow::Result<()> {
        self.try_match_hint_errors(self.handle_command(command).await)
            .await
    }

    async fn handle_command(&self, command: GolemCliCommand) -> anyhow::Result<()> {
        match command {
            GolemCliCommand { subcommand, .. } => match subcommand {
                GolemCliSubcommand::Component { .. } => Ok(()),
                GolemCliSubcommand::Worker { subcommand } => match subcommand {
                    WorkerSubcommand::Invoke {
                        worker_name,
                        function_name,
                        arguments,
                        enqueue,
                    } => {
                        match self.golem_clients().await? {
                            GolemClients::Oss(clients) => {
                                if enqueue {
                                    todo!()
                                } else {
                                    /*clients
                                    .worker
                                    .invoke_and_await_function(
                                        &self.component_id_from_worker_name(worker_name).await?,

                                    )*/
                                    println!("lol");
                                    let component_id =
                                        self.component_id_from_worker_name(&worker_name).await?;
                                    Ok(())
                                }
                            }
                            GolemClients::Cloud(clients) => {
                                todo!()
                            }
                        }
                    }
                },
                GolemCliSubcommand::Api { .. } => Ok(()),
                GolemCliSubcommand::Plugin { .. } => Ok(()),
                GolemCliSubcommand::App { .. } => Ok(()),
                GolemCliSubcommand::Server { .. } => Ok(()),
                GolemCliSubcommand::Cloud { .. } => Ok(()),
                GolemCliSubcommand::Diagnose => Ok(()),
                GolemCliSubcommand::Completion => Ok(()),
            },
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
                for language in GuestLanguage::iter() {
                    eprintln!("  - {}", language);
                    eprintln!("    - default..");
                    eprintln!("    - other");
                }
            }
            GolemCliCommandPartialMatch::WorkerInvokeMissingWorkerName => {
                eprintln!("{}", "\nExisting workers:".underline().bold());
                eprintln!("...");
                eprintln!("To see all workers use.. TODO");
            }
            GolemCliCommandPartialMatch::WorkerInvokeMissingFunctionName { worker_name } => {
                eprintln!(
                    "\n{}",
                    format!("Available functions for {}:", worker_name)
                        .underline()
                        .bold()
                );
                eprintln!("...")
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

    async fn golem_clients(&self) -> anyhow::Result<&GolemClients> {
        Ok(&self.ctx.clients().await?.golem)
    }

    async fn component_id_from_worker_name(
        &self,
        worker_name: &WorkerName,
    ) -> anyhow::Result<Uuid> {
        Err(HintError::WorkerNotFound(worker_name.clone()))?
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
