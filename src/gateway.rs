mod certificate;
mod definition;
mod deployment;
mod domain;
mod healthcheck;

use async_trait::async_trait;
use clap::Subcommand;
use golem_gateway_client::apis::configuration::Configuration;

use crate::clients::gateway::certificate::CertificateClientLive;
use crate::clients::gateway::definition::DefinitionClientLive;
use crate::clients::gateway::deployment::DeploymentClientLive;
use crate::clients::gateway::domain::DomainClientLive;
use crate::clients::gateway::healthcheck::HealthcheckClientLive;
use crate::clients::project::ProjectClient;
use crate::clients::CloudAuthentication;
use crate::gateway::certificate::{
    CertificateHandler, CertificateHandlerLive, CertificateSubcommand,
};
use crate::gateway::definition::{DefinitionHandler, DefinitionHandlerLive, DefinitionSubcommand};
use crate::gateway::deployment::{DeploymentHandler, DeploymentHandlerLive, DeploymentSubcommand};
use crate::gateway::domain::{DomainHandler, DomainHandlerLive, DomainSubcommand};
use crate::gateway::healthcheck::{HealthcheckHandler, HealthcheckHandlerLive};
use crate::model::{Format, GolemError, GolemResult};

#[derive(Subcommand, Debug)]
#[command()]
pub enum GatewaySubcommand {
    #[command()]
    Certificate {
        #[command(subcommand)]
        subcommand: CertificateSubcommand,
    },
    #[command()]
    Definition {
        #[command(subcommand)]
        subcommand: DefinitionSubcommand,
    },
    #[command()]
    Deployment {
        #[command(subcommand)]
        subcommand: DeploymentSubcommand,
    },
    #[command()]
    Domain {
        #[command(subcommand)]
        subcommand: DomainSubcommand,
    },
    #[command()]
    Healthcheck {},
}

#[async_trait]
pub trait GatewayHandler {
    async fn handle(
        &self,
        format: Format,
        token: &CloudAuthentication,
        subcommand: GatewaySubcommand,
    ) -> Result<GolemResult, GolemError>;
}

pub struct GatewayHandlerLive<'p, P: ProjectClient + Sync + Send> {
    pub base_url: reqwest::Url,
    pub allow_insecure: bool,
    pub projects: &'p P,
}

#[async_trait]
impl<'p, P: ProjectClient + Sync + Send> GatewayHandler for GatewayHandlerLive<'p, P> {
    async fn handle(
        &self,
        format: Format,
        auth: &CloudAuthentication,
        subcommand: GatewaySubcommand,
    ) -> Result<GolemResult, GolemError> {
        let mut builder = reqwest::Client::builder();
        if self.allow_insecure {
            builder = builder.danger_accept_invalid_certs(true);
        }
        let client = builder.connection_verbose(true).build()?;

        let mut base_url_string = self.base_url.to_string();

        if base_url_string.pop() != Some('/') {
            base_url_string = self.base_url.to_string();
        }

        let configuration = Configuration {
            base_path: base_url_string,
            user_agent: None,
            client,
            basic_auth: None,
            oauth_access_token: None,
            bearer_access_token: Some(auth.0.secret.value.to_string()),
            api_key: None,
        };

        let healthcheck_client = HealthcheckClientLive {
            configuration: configuration.clone(),
        };
        let healthcheck_srv = HealthcheckHandlerLive {
            healthcheck: healthcheck_client,
        };

        let deployment_client = DeploymentClientLive {
            configuration: configuration.clone(),
        };
        let deployment_srv = DeploymentHandlerLive {
            client: deployment_client,
            projects: self.projects,
        };

        let definition_client = DefinitionClientLive {
            configuration: configuration.clone(),
        };
        let definition_srv = DefinitionHandlerLive {
            client: definition_client,
            projects: self.projects,
        };

        let certificate_client = CertificateClientLive {
            configuration: configuration.clone(),
        };
        let certificate_srv = CertificateHandlerLive {
            client: certificate_client,
            projects: self.projects,
        };

        let domain_client = DomainClientLive {
            configuration: configuration.clone(),
        };
        let domain_srv = DomainHandlerLive {
            client: domain_client,
            projects: self.projects,
        };

        match subcommand {
            GatewaySubcommand::Certificate { subcommand } => {
                certificate_srv.handle(subcommand).await
            }
            GatewaySubcommand::Definition { subcommand } => {
                definition_srv.handle(format, subcommand).await
            }
            GatewaySubcommand::Deployment { subcommand } => deployment_srv.handle(subcommand).await,
            GatewaySubcommand::Domain { subcommand } => domain_srv.handle(subcommand).await,
            GatewaySubcommand::Healthcheck {} => healthcheck_srv.handle().await,
        }
    }
}
