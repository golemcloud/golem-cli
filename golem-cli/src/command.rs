use crate::command::api::ApiSubcommand;
use crate::command::app::AppSubcommand;
use crate::command::cloud::CloudSubcommand;
use crate::command::component::ComponentSubcommand;
use crate::command::plugin::PluginSubcommand;
use crate::command::server::ServerSubcommand;
use crate::command::worker::WorkerSubcommand;
use crate::config::ProfileName;
use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{self, CommandFactory, FromArgMatches, Subcommand};
use clap::{Args, Parser};
use clap_verbosity_flag::Verbosity;
use golem_wasm_rpc_stubgen::log::LogColorize;
use lenient_bool::LenientBool;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct GolemCliCommand {
    #[command(flatten)]
    pub global_flags: GolemCliGlobalFlags,

    #[clap(subcommand)]
    pub subcommand: GolemCliSubcommand,
}

#[derive(Debug, Default, Args)]
pub struct GolemCliGlobalFlags {
    /// Select Golem profile
    #[arg(long, short, global = true)]
    pub profile: Option<ProfileName>,

    /// Custom path to the root application manifest (golem.yaml)
    #[arg(long, short, global = true)]
    pub app_manifest_path: Option<PathBuf>,

    /// Select build profile
    #[arg(long, short, global = true)]
    pub build_profile: Option<String>,

    /// Custom path to the config directory (defaults to $HOME/.golem)
    #[arg(long, short, global = true)]
    pub config_dir: Option<PathBuf>,

    #[command(flatten)]
    pub verbosity: Verbosity,

    #[arg(skip)]
    pub wasm_rpc_path: Option<PathBuf>,

    #[arg(skip)]
    pub wasm_rpc_version: Option<String>,

    #[arg(skip)]
    pub wasm_rpc_offline: bool,
}

impl GolemCliGlobalFlags {
    pub fn with_env_overrides(mut self) -> GolemCliGlobalFlags {
        if self.app_manifest_path.is_none() {
            if let Ok(app_manifest_path) = std::env::var("GOLEM_APP_MANIFEST_PATH") {
                self.app_manifest_path = Some(PathBuf::from(app_manifest_path));
            }
        }

        if self.profile.is_none() {
            if let Ok(profile) = std::env::var("GOLEM_PROFILE") {
                self.profile = Some(profile.into());
            }
        }

        if let Ok(offline) = std::env::var("GOLEM_WASM_RPC_OFFLINE") {
            self.wasm_rpc_offline = offline
                .parse::<LenientBool>()
                .map(|b| b.into())
                .unwrap_or_default()
        }

        if self.wasm_rpc_path.is_none() {
            if let Ok(wasm_rpc_path) = std::env::var("GOLEM_WASM_RPC_PATH") {
                self.wasm_rpc_path = Some(PathBuf::from(wasm_rpc_path));
            }
        }

        if self.wasm_rpc_version.is_none() {
            if let Ok(version) = std::env::var("GOLEM_WASM_RPC_VERSION") {
                self.wasm_rpc_version = Some(version);
            }
        }

        self
    }

    pub fn config_dir(&self) -> PathBuf {
        self.config_dir
            .clone()
            .unwrap_or_else(|| dirs::home_dir().unwrap().join(".golem"))
    }
}

#[derive(Debug, Default, Parser)]
#[command(ignore_errors = true)]
pub struct GolemCliFallbackCommand {
    #[command(flatten)]
    pub global_flags: GolemCliGlobalFlags,

    pub positional_args: Vec<String>,
}

impl GolemCliCommand {
    pub fn try_parse_from_lenient<I, T>(
        iterator: I,
        with_env_overrides: bool,
    ) -> GolemCliCommandParseResult
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let args = iterator
            .into_iter()
            .map(|arg| arg.into())
            .collect::<Vec<OsString>>();

        match GolemCliCommand::try_parse_from(&args) {
            Ok(mut command) => {
                if with_env_overrides {
                    command.global_flags = command.global_flags.with_env_overrides()
                }
                GolemCliCommandParseResult::FullMatch(command)
            }
            Err(error) => {
                let fallback_command = {
                    let mut fallback_command =
                        GolemCliFallbackCommand::try_parse_from(args).unwrap_or_default();
                    if with_env_overrides {
                        fallback_command.global_flags =
                            fallback_command.global_flags.with_env_overrides()
                    }
                    fallback_command
                };

                let partial_match = match error.kind() {
                    ErrorKind::MissingRequiredArgument => {
                        error.context().find_map(|context| match context {
                            (ContextKind::InvalidArg, ContextValue::Strings(args)) => {
                                Self::match_invalid_arg(
                                    &fallback_command.positional_args,
                                    args,
                                    &Self::invalid_arg_matchers(),
                                )
                            }
                            _ => None,
                        })
                    }
                    ErrorKind::MissingSubcommand if fallback_command.positional_args == ["app"] => {
                        Some(GolemCliCommandPartialMatch::AppMissingSubcommandHelp)
                    }
                    ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
                        if fallback_command.positional_args == ["app"] =>
                    {
                        Some(GolemCliCommandPartialMatch::AppMissingSubcommandHelp)
                    }
                    _ => None,
                };

                match partial_match {
                    Some(partial_match) => GolemCliCommandParseResult::ErrorWithPartialMatch {
                        error,
                        fallback_command,
                        partial_match,
                    },
                    None => GolemCliCommandParseResult::Error {
                        error,
                        fallback_command,
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
        positional_args: &[String],
        error_context_args: &[String],
        matchers: &[InvalidArgMatcher],
    ) -> Option<GolemCliCommandPartialMatch> {
        let command = Self::command();

        let positional_args = positional_args
            .iter()
            .map(|str| str.as_str())
            .collect::<Vec<_>>();

        for matcher in matchers {
            if positional_args.len() < matcher.subcommands.len() {
                continue;
            }

            let missing_arg_error_name =
                format!("<{}>", matcher.missing_positional_arg.to_uppercase());
            let missing_args_error_name = format!("{}...", missing_arg_error_name);
            if !error_context_args.contains(&missing_arg_error_name)
                && !error_context_args.contains(&missing_args_error_name)
            {
                continue;
            }

            if !positional_args.starts_with(&matcher.subcommands) {
                continue;
            }

            let mut command = &command;
            for subcommand in &matcher.subcommands {
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
        fallback_command: GolemCliFallbackCommand,
        partial_match: GolemCliCommandPartialMatch,
    },
    Error {
        error: clap::Error,
        fallback_command: GolemCliFallbackCommand,
    },
}

#[derive(Debug)]
pub enum GolemCliCommandPartialMatch {
    AppNewMissingLanguage,
    AppMissingSubcommandHelp,
    ComponentNewMissingLanguage,
    WorkerInvokeMissingWorkerName,
    WorkerInvokeMissingFunctionName { worker_name: String },
}

#[derive(Debug, Subcommand)]
pub enum GolemCliSubcommand {
    #[command(alias = "application")]
    /// Build, deploy application
    App {
        #[clap(subcommand)]
        subcommand: AppSubcommand,
    },
    /// Build, deploy and manage components
    Component {
        #[clap(subcommand)]
        subcommand: ComponentSubcommand,
    },
    /// Invoke and manage workers
    Worker {
        #[clap(subcommand)]
        subcommand: WorkerSubcommand,
    },
    /// Manage API gateway objects
    Api {
        #[clap(subcommand)]
        subcommand: ApiSubcommand,
    },
    /// Manage plugins
    Plugin {
        #[clap(subcommand)]
        subcommand: PluginSubcommand,
    },
    // TODO: add feature for server
    /// Run and manage the local Golem server
    Server {
        #[clap(subcommand)]
        subcommand: ServerSubcommand,
    },
    /// Manage Golem Cloud accounts and projects
    Cloud {
        #[clap(subcommand)]
        subcommand: CloudSubcommand,
    },
    /// Diagnose possible problems
    Diagnose,
    /// Generate shell completion
    Completion,
}

pub mod shared_args {
    use crate::model::ComponentName;
    use clap::Args;
    use golem_examples::model::GuestLanguage;

    // TODO: move names to model
    pub type ApplicationName = String;
    pub type ProjectName = String;
    pub type NewComponentName = String;
    pub type WorkerFunctionArgument = String;
    pub type WorkerFunctionName = String;

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
    pub struct LanguageArg {
        #[clap(long, short)]
        pub language: GuestLanguage,
    }

    #[derive(Debug, Args)]
    pub struct LanguagePositionalArg {
        pub language: GuestLanguage,
    }

    #[derive(Debug, Args)]
    pub struct LanguagesPositionalArg {
        #[clap(required = true)]
        pub language: Vec<GuestLanguage>,
    }

    #[derive(Debug, Args)]
    pub struct ForceBuildArg {
        /// When set to true will skip modification time based up-to-date checks, defaults to false
        #[clap(long, short, default_value = "false")]
        force_build: bool,
    }
}

pub mod app {
    use crate::command::shared_args::{
        AppOptionalComponentNames, ApplicationName, ForceBuildArg, LanguagesPositionalArg,
    };
    use clap::Subcommand;
    use golem_wasm_rpc_stubgen::model::app::AppBuildStep;

    #[derive(Debug, Subcommand)]
    pub enum AppSubcommand {
        /// Create new application
        New {
            application_name: ApplicationName,
            #[command(flatten)]
            language: LanguagesPositionalArg,
        },
        /// Build all or selected components in the application
        Build {
            #[command(flatten)]
            component_name: AppOptionalComponentNames,
            /// Select specific build step(s)
            #[clap(long, short)]
            step: Vec<AppBuildStep>,
            #[command(flatten)]
            force_build: ForceBuildArg,
        },
        /// Deploy all or selected components in the application, includes building
        Deploy {
            #[command(flatten)]
            component_name: AppOptionalComponentNames,
            #[command(flatten)]
            force_build: ForceBuildArg,
        },
        /// Clean all components in the application or by selection
        Clean {
            #[command(flatten)]
            component_name: AppOptionalComponentNames,
        },
        /// Run custom command
        #[clap(external_subcommand)]
        CustomCommand(Vec<String>),
    }
}

pub mod component {
    use crate::command::shared_args::ComponentOptionalComponentNames;
    use crate::command::shared_args::{LanguagePositionalArg, NewComponentName};
    use clap::Subcommand;

    #[derive(Debug, Subcommand)]
    pub enum ComponentSubcommand {
        /// Create new component in the application
        New {
            component_name: NewComponentName,
            #[command(flatten)]
            language: LanguagePositionalArg,
            /// Select template
            template_name: Option<String>,
        },
        /// Build component(s) based on the current directory or by selection
        Build {
            #[command(flatten)]
            component_name: ComponentOptionalComponentNames,
        },
        /// Deploy component(s) based on the current directory or by selection
        Deploy {
            #[command(flatten)]
            component_name: ComponentOptionalComponentNames,
        },
        /// Clean component(s) based on the current directory or by selection
        Clean {
            #[command(flatten)]
            component_name: ComponentOptionalComponentNames,
        },
    }
}

pub mod worker {
    use crate::command::shared_args::{WorkerFunctionArgument, WorkerFunctionName};
    use crate::model::WorkerName;
    use clap::Subcommand;

    #[derive(Debug, Subcommand)]
    pub enum WorkerSubcommand {
        Invoke {
            worker_name: WorkerName,
            function_name: WorkerFunctionName,
            arguments: Vec<WorkerFunctionArgument>,
            #[clap(long, short, default_value = "false")]
            enqueue: bool,
        },
    }
}

pub mod api {
    use crate::command::api::cloud::ApiCloudSubcommand;
    use crate::command::api::definition::ApiDefinitionSubcommand;
    use crate::command::api::deployment::ApiDeploymentSubcommand;
    use crate::command::api::security_scheme::ApiSecuritySchemeSubcommand;
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
        use crate::command::api::cloud::certificate::ApiCertificateSubcommand;
        use crate::command::api::cloud::domain::ApiDomainSubcommand;
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
    use crate::command::cloud::account::AccountSubcommand;
    use crate::command::cloud::auth_token::AuthTokenSubcommand;
    use crate::command::cloud::project::ProjectSubcommand;
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

pub mod server {
    use clap::Subcommand;
    use std::path::PathBuf;

    #[derive(Debug, Subcommand)]
    pub enum ServerSubcommand {
        /// Run golem server for local development
        Run {
            /// Address to serve the main API on
            #[clap(long, default_value = "0.0.0.0")]
            router_addr: String,

            /// Port to serve the main API on
            #[clap(long, default_value_t = 9881)]
            router_port: u16,

            /// Port to serve custom requests on
            #[clap(long, default_value_t = 9006)]
            custom_request_port: u16,

            /// Directory to store data in. Defaults to $XDG_STATE_HOME/golem
            #[clap(long)]
            data_dir: Option<PathBuf>,

            /// Clean the data directory before starting
            #[clap(long, default_value = "false")]
            clean: bool,
        },
        /// Clean the local server data directory
        Clean,
    }
}
