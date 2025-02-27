use colored::Colorize;
use golem_cli::command_v_1_2::worker::WorkerSubcommand;
use golem_cli::command_v_1_2::{
    GolemCliCommand, GolemCliCommandParseResult, GolemCliCommandPartialMatch, GolemCliSubcommand,
};
use golem_cli::init_tracing;
use golem_examples::model::GuestLanguage;
use strum::IntoEnumIterator;
use tracing::{debug, Level};

fn main() {
    match GolemCliCommand::try_parse_from_lenient(std::env::args_os()) {
        GolemCliCommandParseResult::FullMatch(command) => {
            init_tracing(command.global_flag.verbosity);

            match command {
                GolemCliCommand { subcommand, .. } => match subcommand {
                    GolemCliSubcommand::Component { .. } => {}
                    GolemCliSubcommand::Worker { subcommand } => match subcommand {
                        WorkerSubcommand::Invoke {
                            worker_name,
                            function_name,
                            arguments,
                        } => {
                            println!("Invoke");
                            println!("worker: {:?}", worker_name);
                            println!("function: {:?}", function_name);
                            println!("arguments: {:?}", arguments);
                        }
                    },
                    GolemCliSubcommand::Api { .. } => {}
                    GolemCliSubcommand::Plugin { .. } => {}
                    GolemCliSubcommand::App { .. } => {}
                    GolemCliSubcommand::Cloud { .. } => {}
                    GolemCliSubcommand::Diagnose => {}
                    GolemCliSubcommand::Completion => {}
                },
            }
        }
        GolemCliCommandParseResult::ErrorWithPartialMatch {
            error,
            global_flags,
            partial_match,
        } => {
            init_tracing(global_flags.verbosity);

            error.print().unwrap();

            match partial_match {
                GolemCliCommandPartialMatch::AppNewMissingLanguage
                | GolemCliCommandPartialMatch::ComponentNewMissingLanguage => {
                    eprintln!("{}", "\nAvailable languages:".underline().bold());
                    for language in GuestLanguage::iter() {
                        eprintln!("  - {}", language);
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

            std::process::exit(error.exit_code());
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

            std::process::exit(error.exit_code());
        }
    }
}
