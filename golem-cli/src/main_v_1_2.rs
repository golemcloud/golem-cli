use clap::error::{ContextKind, ContextValue};
use clap::{CommandFactory, Parser};
use colored::Colorize;
use golem_cli::command_v_1_2::worker::WorkerSubcommand;
use golem_cli::command_v_1_2::{GolemCliCommand, GolemCliSubcommand};
use golem_examples::model::GuestLanguage;
use std::collections::HashMap;
use strum::IntoEnumIterator;
use tracing::debug;

fn main() {
    match GolemCliCommand::try_parse() {
        Ok(command) => run_command(command),
        Err(error) => {
            // TODO: add a fallback parser for logging and profile
            error.print().unwrap();
            enrich_clap_error(&error);
            std::process::exit(error.exit_code());
        }
    };
}

fn run_command(command: GolemCliCommand) {
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

fn enrich_clap_error(error: &clap::Error) {
    for context in error.context() {
        match &context {
            (ContextKind::InvalidArg, ContextValue::Strings(args)) => {
                handle_invalid_arg(
                    args,
                    &[
                        (&["app", "new"], &[], "language", |_| {
                            eprintln!("{}", "\nAvailable languages:".underline().bold());
                            for language in GuestLanguage::iter() {
                                eprintln!("  - {}", language);
                            }
                        }),
                        (&["component", "new"], &[], "language", |_| {
                            eprintln!("{}", "\nAvailable languages:".underline().bold());
                            for language in GuestLanguage::iter() {
                                eprintln!("  - {}", language);
                            }
                        }),
                        (&["worker", "invoke"], &[], "worker_name", |_| {
                            eprintln!("{}", "\nExisting workers:".underline().bold());
                            eprintln!("...");
                            eprintln!("To see all workers use.. TODO");
                        }),
                        (
                            &["worker", "invoke"],
                            &["worker_name"],
                            "function_name",
                            |matched_args| {
                                eprintln!(
                                    "\n{}",
                                    format!("Available functions for {}:", matched_args[0])
                                        .underline()
                                        .bold()
                                );
                                eprintln!("...")
                            },
                        ),
                    ],
                );
            }
            _ => {
                debug!(
                    context_kind = context.0.as_str(),
                    context_value = context.1.to_string(),
                    "clap error context",
                );
            }
        }
    }
}

// TODO: add doc about how matchers are structured
fn handle_invalid_arg(
    error_context_args: &[String],
    matchers: &[(&[&str], &[&str], &str, fn(Vec<String>))],
) {
    let command = GolemCliCommand::command();

    let positional_args = std::env::args()
        .skip(1)
        .filter(|arg| !arg.starts_with('-'))
        .collect::<Vec<_>>();

    let positional_args = positional_args
        .iter()
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();

    for (subcommands, expected_arg_names, missing_arg_name, action) in matchers {
        let missing_arg_error_name = format!("<{}>", missing_arg_name.to_uppercase());
        if positional_args.len() < subcommands.len() {
            continue;
        }
        if !error_context_args.contains(&missing_arg_error_name) {
            continue;
        }
        if !positional_args.starts_with(subcommands) {
            continue;
        }

        let mut command = &command;
        for subcommand in *subcommands {
            // TODO: unit test for unwrap (e.g. let's add sample test for all the matchers)
            command = command.find_subcommand(subcommand).unwrap();
        }
        let positional_arg_ids_to_idx = command
            .get_arguments()
            .filter(|arg| arg.is_positional())
            .enumerate()
            .map(|(idx, arg)| (arg.get_id().to_string(), idx))
            .collect::<HashMap<_, _>>();

        let mut found_args = Vec::<String>::with_capacity(expected_arg_names.len());
        for expected_arg_name in *expected_arg_names {
            let Some(idx) = positional_arg_ids_to_idx.get(*expected_arg_name) else {
                break;
            };
            let Some(arg_value) = positional_args.get(subcommands.len() + *idx) else {
                break;
            };
            found_args.push(arg_value.to_string());
        }
        if found_args.len() != expected_arg_names.len() {
            continue;
        } else {
            action(found_args);
            return;
        }
    }
}
