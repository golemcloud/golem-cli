use crate::command_v_1_2::api::ApiSubcommand;
use crate::command_v_1_2::app::AppSubcommand;
use crate::command_v_1_2::cloud::CloudSubcommand;
use crate::command_v_1_2::component::ComponentSubcommand;
use crate::command_v_1_2::plugin::PluginSubcommand;
use crate::command_v_1_2::worker::WorkerSubcommand;
use clap::error::{ContextKind, ContextValue};
use clap::{self, CommandFactory, Subcommand};
use clap::{Args, Parser};
use clap_verbosity_flag::Verbosity;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct GolemCliCommand {
    #[command(flatten)]
    pub global_flag: GolemCliGlobalFlags,

    #[clap(subcommand)]
    pub subcommand: GolemCliSubcommand,
}

#[derive(Debug, Default, Args)]
pub struct GolemCliGlobalFlags {
    #[command(flatten)]
    pub verbosity: Verbosity,

    /// Custom path to the root application manifest (golem.yaml)
    #[arg(long, short, global = true)]
    pub app_manifest_path: Option<PathBuf>,
}

#[derive(Debug, Default, Parser)]
#[command(ignore_errors = true)]
pub struct GolemCliFallbackCommand {
    #[command(flatten)]
    pub global_flags: GolemCliGlobalFlags,
}

impl GolemCliCommand {
    pub fn try_parse_from_lenient<I, T>(iterator: I) -> GolemCliCommandParseResult
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let args = iterator
            .into_iter()
            .map(|arg| arg.into())
            .collect::<Vec<OsString>>();

        match GolemCliCommand::try_parse_from(&args) {
            Ok(command) => GolemCliCommandParseResult::FullMatch(command),
            Err(error) => {
                // TODO: there are still cases where this will fail (e.g. unknown flags before known ones)
                let fallback_global_flags = GolemCliFallbackCommand::try_parse_from(args)
                    .unwrap_or_default()
                    .global_flags;

                let invalid_arg_matchers = Self::invalid_arg_matchers();
                let partial_match = error.context().find_map(|context| match context {
                    (ContextKind::InvalidArg, ContextValue::Strings(args)) => {
                        Self::match_invalid_arg(args, &invalid_arg_matchers)
                    }
                    _ => None,
                });

                match partial_match {
                    Some(partial_match) => GolemCliCommandParseResult::ErrorWithPartialMatch {
                        error,
                        global_flags: fallback_global_flags,
                        partial_match,
                    },
                    None => GolemCliCommandParseResult::Error {
                        error,
                        global_flags: fallback_global_flags,
                    },
                }
            }
        }
    }

    // TODO: unit test for checking validity of subcommands and arg names
    fn invalid_arg_matchers() -> Vec<InvalidArgMatcher> {
        vec![
            InvalidArgMatcher {
                subcommands: vec!["app", "new"],
                found_positional_args: vec![],
                missing_positional_arg: "language",
                to_partial_match: |_| GolemCliCommandPartialMatch::AppNewMissingLanguage,
            },
            InvalidArgMatcher {
                subcommands: vec!["component", "new"],
                found_positional_args: vec![],
                missing_positional_arg: "language",
                to_partial_match: |_| GolemCliCommandPartialMatch::ComponentNewMissingLanguage,
            },
            InvalidArgMatcher {
                subcommands: vec!["worker", "invoke"],
                found_positional_args: vec![],
                missing_positional_arg: "worker_name",
                to_partial_match: |_| GolemCliCommandPartialMatch::WorkerInvokeMissingWorkerName,
            },
            InvalidArgMatcher {
                subcommands: vec!["worker", "invoke"],
                found_positional_args: vec!["worker_name"],
                missing_positional_arg: "function_name",
                to_partial_match: |args| {
                    GolemCliCommandPartialMatch::WorkerInvokeMissingFunctionName {
                        worker_name: args[0].clone(),
                    }
                },
            },
        ]
    }

    fn match_invalid_arg(
        error_context_args: &[String],
        matchers: &[InvalidArgMatcher],
    ) -> Option<GolemCliCommandPartialMatch> {
        let command = Self::command();

        let positional_args = std::env::args()
            .skip(1)
            .filter(|arg| !arg.starts_with('-'))
            .collect::<Vec<_>>();

        let positional_args = positional_args
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>();

        for matcher in matchers {
            let missing_arg_error_name =
                format!("<{}>", matcher.missing_positional_arg.to_uppercase());
            if positional_args.len() < matcher.subcommands.len() {
                continue;
            }
            if !error_context_args.contains(&missing_arg_error_name) {
                continue;
            }
            if !positional_args.starts_with(&matcher.subcommands) {
                continue;
            }

            let mut command = &command;
            for subcommand in &matcher.subcommands {
                // TODO: unit test for unwrap (e.g. let's add sample test for all the matchers)
                command = command.find_subcommand(subcommand).unwrap();
            }
            let positional_arg_ids_to_idx = command
                .get_arguments()
                .filter(|arg| arg.is_positional())
                .enumerate()
                .map(|(idx, arg)| (arg.get_id().to_string(), idx))
                .collect::<HashMap<_, _>>();

            let mut found_args = Vec::<String>::with_capacity(matcher.found_positional_args.len());
            for expected_arg_name in &matcher.found_positional_args {
                let Some(idx) = positional_arg_ids_to_idx.get(*expected_arg_name) else {
                    break;
                };
                let Some(arg_value) = positional_args.get(matcher.subcommands.len() + *idx) else {
                    break;
                };
                found_args.push(arg_value.to_string());
            }
            if found_args.len() == matcher.found_positional_args.len() {
                return Some((matcher.to_partial_match)(found_args));
            }
        }

        None
    }
}

struct InvalidArgMatcher {
    pub subcommands: Vec<&'static str>,
    pub found_positional_args: Vec<&'static str>,
    pub missing_positional_arg: &'static str,
    pub to_partial_match: fn(Vec<String>) -> GolemCliCommandPartialMatch,
}

pub enum GolemCliCommandParseResult {
    FullMatch(GolemCliCommand),
    ErrorWithPartialMatch {
        error: clap::Error,
        global_flags: GolemCliGlobalFlags,
        partial_match: GolemCliCommandPartialMatch,
    },
    Error {
        error: clap::Error,
        global_flags: GolemCliGlobalFlags,
    },
}

pub enum GolemCliCommandPartialMatch {
    AppNewMissingLanguage,
    ComponentNewMissingLanguage,
    WorkerInvokeMissingWorkerName,
    WorkerInvokeMissingFunctionName { worker_name: String },
}

#[derive(Debug, Subcommand)]
pub enum GolemCliSubcommand {
    #[command(alias = "application")]
    App {
        #[clap(subcommand)]
        subcommand: AppSubcommand,
    },
    Component {
        #[clap(subcommand)]
        subcommand: ComponentSubcommand,
    },
    Worker {
        #[clap(subcommand)]
        subcommand: WorkerSubcommand,
    },
    Api {
        #[clap(subcommand)]
        subcommand: ApiSubcommand,
    },
    Plugin {
        #[clap(subcommand)]
        subcommand: PluginSubcommand,
    },
    Cloud {
        #[clap(subcommand)]
        subcommand: CloudSubcommand,
    },
    Diagnose,
    Completion,
}

pub mod shared_args {
    use clap::Args;
    use golem_examples::model::GuestLanguage;

    pub type ApplicationName = String;
    pub type ComponentName = String;
    pub type ProjectName = String;
    pub type NewComponentName = String;
    pub type WorkerFunctionArgument = String;
    pub type WorkerFunctionName = String;
    pub type WorkerName = String;

    #[derive(Debug, Args)]
    pub struct ComponentOptionalComponentNames {
        /// Optional component names, if not specified components are selected based on the current directory
        component_name: Vec<ComponentName>,
    }

    #[derive(Debug, Args)]
    pub struct AppOptionalComponentNames {
        /// Optional component names, if not specified all components are selected.
        component_name: Vec<ComponentName>,
    }

    #[derive(Debug, Args)]
    pub struct BuildProfile {
        /// Selects a build profile
        #[clap(long, short)]
        pub build_profile: Option<String>,
    }

    #[derive(Debug, Args)]
    pub struct LanguageArg {
        #[clap(long, short)]
        pub language: GuestLanguage,
    }

    #[derive(Debug, Args)]
    pub struct LanguagePositionalArg {
        pub language: GuestLanguage,
    }
}

pub mod app {
    use crate::command_v_1_2::shared_args::{
        AppOptionalComponentNames, ApplicationName, BuildProfile, LanguagePositionalArg,
    };
    use clap::Subcommand;

    #[derive(Debug, Subcommand)]
    pub enum AppSubcommand {
        New {
            application_name: ApplicationName,
            #[command(flatten)]
            language: LanguagePositionalArg,
        },
        Build {
            #[command(flatten)]
            component_name: AppOptionalComponentNames,
            #[command(flatten)]
            build_profile: BuildProfile,
        },
        Clean {
            #[command(flatten)]
            component_name: AppOptionalComponentNames,
            #[command(flatten)]
            build_profile: BuildProfile,
        },
        Deploy {
            #[command(flatten)]
            component_name: AppOptionalComponentNames,
            #[command(flatten)]
            build_profile: BuildProfile,
        },
    }
}

pub mod component {
    use crate::command_v_1_2::shared_args::{BuildProfile, ComponentOptionalComponentNames};
    use crate::command_v_1_2::shared_args::{LanguagePositionalArg, NewComponentName};
    use clap::Subcommand;

    #[derive(Debug, Subcommand)]
    pub enum ComponentSubcommand {
        New {
            component_name: NewComponentName,
            #[command(flatten)]
            language: LanguagePositionalArg,
        },
        Build {
            #[command(flatten)]
            component_name: ComponentOptionalComponentNames,
            #[command(flatten)]
            build_profile: BuildProfile,
        },
        Clean {
            #[command(flatten)]
            component_name: ComponentOptionalComponentNames,
            #[command(flatten)]
            build_profile: BuildProfile,
        },
        Deploy {
            #[command(flatten)]
            component_name: ComponentOptionalComponentNames,
            #[command(flatten)]
            build_profile: BuildProfile,
        },
    }
}

pub mod worker {
    use crate::command_v_1_2::shared_args::{
        WorkerFunctionArgument, WorkerFunctionName, WorkerName,
    };
    use clap::Subcommand;

    #[derive(Debug, Subcommand)]
    pub enum WorkerSubcommand {
        Invoke {
            worker_name: WorkerName,
            function_name: WorkerFunctionName,
            arguments: Vec<WorkerFunctionArgument>,
        },
    }
}

pub mod api {
    use crate::command_v_1_2::api::cloud::ApiCloudSubcommand;
    use crate::command_v_1_2::api::definition::ApiDefinitionSubcommand;
    use crate::command_v_1_2::api::deployment::ApiDeploymentSubcommand;
    use crate::command_v_1_2::api::security_scheme::ApiSecuritySchemeSubcommand;
    use clap::Subcommand;

    #[derive(Debug, Subcommand)]
    pub enum ApiSubcommand {
        Definition {
            #[clap(subcommand)]
            subcommand: ApiDefinitionSubcommand,
        },
        Deployment {
            #[clap(subcommand)]
            subcommand: ApiDeploymentSubcommand,
        },
        SecurityScheme {
            #[clap(subcommand)]
            subcommand: ApiSecuritySchemeSubcommand,
        },
        Cloud {
            #[clap(subcommand)]
            subcommand: ApiCloudSubcommand,
        },
    }

    pub mod definition {
        use clap::Subcommand;

        #[derive(Debug, Subcommand)]
        pub enum ApiDefinitionSubcommand {}
    }

    pub mod deployment {
        use clap::Subcommand;

        #[derive(Debug, Subcommand)]
        pub enum ApiDeploymentSubcommand {}
    }

    pub mod security_scheme {
        use clap::Subcommand;

        #[derive(Debug, Subcommand)]
        pub enum ApiSecuritySchemeSubcommand {}
    }

    pub mod cloud {
        use crate::command_v_1_2::api::cloud::certificate::ApiCertificateSubcommand;
        use crate::command_v_1_2::api::cloud::domain::ApiDomainSubcommand;
        use clap::Subcommand;

        #[derive(Debug, Subcommand)]
        pub enum ApiCloudSubcommand {
            Domain {
                #[clap(subcommand)]
                subcommand: ApiDomainSubcommand,
            },
            Certificate {
                #[clap(subcommand)]
                subcommand: ApiCertificateSubcommand,
            },
        }

        pub mod domain {
            use clap::Subcommand;

            #[derive(Debug, Subcommand)]
            pub enum ApiDomainSubcommand {}
        }

        pub mod certificate {
            use clap::Subcommand;

            #[derive(Debug, Subcommand)]
            pub enum ApiCertificateSubcommand {}
        }
    }
}

pub mod plugin {
    use clap::Subcommand;

    #[derive(Debug, Subcommand)]
    pub enum PluginSubcommand {}
}

pub mod cloud {
    use crate::command_v_1_2::cloud::account::AccountSubcommand;
    use crate::command_v_1_2::cloud::auth_token::AuthTokenSubcommand;
    use crate::command_v_1_2::cloud::project::ProjectSubcommand;
    use clap::Subcommand;

    #[derive(Debug, Subcommand)]
    pub enum CloudSubcommand {
        AuthToken {
            #[clap(subcommand)]
            subcommand: AuthTokenSubcommand,
        },
        Account {
            #[clap(subcommand)]
            subcommand: AccountSubcommand,
        },
        Project {
            #[clap(subcommand)]
            subcommand: ProjectSubcommand,
        },
    }

    pub mod auth_token {
        use clap::Subcommand;

        #[derive(Debug, Subcommand)]
        pub enum AuthTokenSubcommand {}
    }

    pub mod account {
        use clap::Subcommand;

        #[derive(Debug, Subcommand)]
        pub enum AccountSubcommand {}
    }

    pub mod project {
        use clap::Subcommand;

        #[derive(Debug, Subcommand)]
        pub enum ProjectSubcommand {}
    }
}
