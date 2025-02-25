use crate::command_v_1_2::api::ApiSubcommand;
use crate::command_v_1_2::app::AppSubcommand;
use crate::command_v_1_2::cloud::CloudSubcommand;
use crate::command_v_1_2::component::ComponentSubcommand;
use crate::command_v_1_2::plugin::PluginSubcommand;
use crate::command_v_1_2::worker::WorkerSubcommand;
use clap::Parser;
use clap::{self, Subcommand};
use clap_verbosity_flag::Verbosity;
use std::path::PathBuf;

// TODO: let's pull up custom manifest location here, and only allow one to be defined
#[derive(Debug, Parser)]
pub struct GolemCliCommand {
    #[command(flatten)]
    pub verbosity: Verbosity,

    /// Custom path to the root application manifest (golem.yaml)
    #[arg(long, short, global = true)]
    pub app_manifest_path: Option<PathBuf>,

    #[clap(subcommand)]
    pub subcommand: GolemCliSubcommand,
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
