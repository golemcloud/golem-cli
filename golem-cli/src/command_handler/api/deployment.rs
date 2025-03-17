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

use crate::command::api::deployment::ApiDeploymentSubcommand;
use crate::command::shared_args::ProjectNameOptionalArg;
use crate::command_handler::Handlers;
use crate::context::{Context, GolemClients};
use crate::error::service::AnyhowMapServiceError;
use crate::error::NonSuccessfulExit;
use crate::model::text::fmt::log_error;
use crate::model::{ApiDefinitionId, ApiDefinitionIdWithVersion, ApiDeployment};
use anyhow::bail;
use golem_client::api::ApiDeploymentClient as ApiDeploymentClientOss;
use golem_client::model::{
    ApiDefinitionInfo as ApiDefinitionInfoOss, ApiDeploymentRequest as ApiDeploymentRequestOss,
    ApiSite as ApiSiteOss,
};
use golem_cloud_client::api::ApiDeploymentClient as ApiDeploymentCloudOss;
use golem_cloud_client::model::{
    ApiDefinitionInfo as ApiDefinitionInfoCloud, ApiDeploymentRequest as ApiDeploymentRequestCloud,
    ApiSite as ApiSiteCloud,
};
use golem_wasm_rpc_stubgen::log::{log_warn_action, LogColorize};
use std::sync::Arc;

pub struct ApiDeploymentCommandHandler {
    ctx: Arc<Context>,
}

impl ApiDeploymentCommandHandler {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    pub async fn handle_command(&mut self, command: ApiDeploymentSubcommand) -> anyhow::Result<()> {
        match command {
            ApiDeploymentSubcommand::Deploy {
                project,
                definitions,
                host,
                subdomain,
            } => self.deploy(project, definitions, host, subdomain).await,
            ApiDeploymentSubcommand::Get { site } => self.get(site).await,
            ApiDeploymentSubcommand::List {
                project,
                definition,
            } => self.list(project, definition).await,
            ApiDeploymentSubcommand::Delete { site } => self.delete(site).await,
        }
    }

    async fn deploy(
        &self,
        project: ProjectNameOptionalArg,
        api_defs: Vec<ApiDefinitionIdWithVersion>,
        host: String,
        subdomain: Option<String>,
    ) -> anyhow::Result<()> {
        let project = self
            .ctx
            .cloud_project_handler()
            .opt_select_project(None /* TODO: account id */, project.project.as_ref())
            .await?;

        let result: ApiDeployment = match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => clients
                .api_deployment
                .deploy(&ApiDeploymentRequestOss {
                    api_definitions: api_defs
                        .iter()
                        .map(|d| ApiDefinitionInfoOss {
                            id: d.id.0.clone(),
                            version: d.version.0.clone(),
                        })
                        .collect::<Vec<_>>(),
                    site: ApiSiteOss { host, subdomain },
                })
                .await
                .map_service_error()?
                .into(),
            GolemClients::Cloud(clients) => {
                let project = self
                    .ctx
                    .cloud_project_handler()
                    .selected_project_or_default(project)
                    .await?;
                clients
                    .api_deployment
                    .deploy(&ApiDeploymentRequestCloud {
                        project_id: project.project_id.0,
                        api_definitions: api_defs
                            .iter()
                            .map(|d| ApiDefinitionInfoCloud {
                                id: d.id.0.clone(),
                                version: d.version.0.clone(),
                            })
                            .collect::<Vec<_>>(),
                        site: ApiSiteCloud { host, subdomain },
                    })
                    .await
                    .map_service_error()?
                    .into()
            }
        };

        self.ctx.log_handler().log_view(&result);

        Ok(())
    }

    async fn get(&self, site: String) -> anyhow::Result<()> {
        let result: ApiDeployment = match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => clients
                .api_deployment
                .get_deployment(&site)
                .await
                .map_service_error()?
                .into(),
            GolemClients::Cloud(clients) => clients
                .api_deployment
                .get_deployment(&site)
                .await
                .map_service_error()?
                .into(),
        };

        self.ctx.log_handler().log_view(&result);

        Ok(())
    }

    async fn list(
        &self,
        project: ProjectNameOptionalArg,
        definition: Option<ApiDefinitionId>,
    ) -> anyhow::Result<()> {
        let id = definition.as_ref().map(|id| id.0.as_str());

        let project = self
            .ctx
            .cloud_project_handler()
            .opt_select_project(None /* TODO: account id */, project.project.as_ref())
            .await?;

        let result: Vec<ApiDeployment> = match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => clients
                .api_deployment
                .list_deployments(id)
                .await
                .map_service_error()?
                .into_iter()
                .map(ApiDeployment::from)
                .collect::<Vec<_>>(),
            GolemClients::Cloud(clients) => {
                let project = self
                    .ctx
                    .cloud_project_handler()
                    .selected_project_or_default(project)
                    .await?;

                match id {
                    Some(id) => clients
                        .api_deployment
                        .list_deployments(&project.project_id.0, id)
                        .await
                        .map_service_error()?
                        .into_iter()
                        .map(ApiDeployment::from)
                        .collect::<Vec<_>>(),
                    None => {
                        // TODO: update in cloud to allow listing without id
                        log_error("API definition ID for Cloud is required");
                        bail!(NonSuccessfulExit);
                    }
                }
            }
        };

        self.ctx.log_handler().log_view(&result);

        Ok(())
    }

    async fn delete(&self, site: String) -> anyhow::Result<()> {
        match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => clients
                .api_deployment
                .delete_deployment(&site)
                .await
                .map(|_| ())
                .map_service_error()?,
            GolemClients::Cloud(clients) => clients
                .api_deployment
                .delete_deployment(&site)
                .await
                .map(|_| ())
                .map_service_error()?,
        };

        log_warn_action("Deleted", format!("site {}", site.log_color_highlight()));

        Ok(())
    }
}
