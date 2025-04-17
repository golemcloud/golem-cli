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

use crate::command::api::security_scheme::ApiSecuritySchemeSubcommand;
use crate::command::shared_args::ProjectNameOptionalArg;
use crate::command_handler::Handlers;
use crate::context::{Context, GolemClients};
use crate::error::service::AnyhowMapServiceError;
use crate::model::api::{ApiSecurityScheme, IdentityProviderType};
use golem_client::api::ApiSecurityClient as ApiSecurityClientOss;
use golem_client::model::{Provider as ProviderOss, SecuritySchemeData as SecuritySchemeDataOss};
use golem_cloud_client::api::ApiSecurityClient as ApiSecurityClientCloud;
use golem_cloud_client::model::{
    Provider as ProviderCloud, SecuritySchemeData as SecuritySchemeDataCloud,
};
use std::sync::Arc;

pub struct ApiSecuritySchemeCommandHandler {
    ctx: Arc<Context>,
}

impl ApiSecuritySchemeCommandHandler {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    pub async fn handle_command(
        &mut self,
        command: ApiSecuritySchemeSubcommand,
    ) -> anyhow::Result<()> {
        match command {
            ApiSecuritySchemeSubcommand::Create {
                project,
                security_scheme_id,
                provider_type,
                client_id,
                client_secret,
                scope,
                redirect_url,
            } => {
                self.cmd_create(
                    project,
                    security_scheme_id,
                    provider_type,
                    client_id,
                    client_secret,
                    scope,
                    redirect_url,
                )
                .await
            }
            ApiSecuritySchemeSubcommand::Get {
                project,
                security_scheme_id,
            } => self.cmd_get(project, security_scheme_id).await,
        }
    }

    async fn cmd_create(
        &self,
        project: ProjectNameOptionalArg,
        scheme_identifier: String,
        provider_type: IdentityProviderType,
        client_id: String,
        client_secret: String,
        scopes: Vec<String>,
        redirect_url: String,
    ) -> anyhow::Result<()> {
        let project = self
            .ctx
            .cloud_project_handler()
            .opt_select_project(None /* TODO: account id */, project.project.as_ref())
            .await?;

        let result: ApiSecurityScheme = match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => clients
                .api_security
                .create(&SecuritySchemeDataOss {
                    provider_type: match provider_type {
                        IdentityProviderType::Google => ProviderOss::Google,
                        IdentityProviderType::Facebook => ProviderOss::Facebook,
                        IdentityProviderType::Gitlab => ProviderOss::Gitlab,
                        IdentityProviderType::Microsoft => ProviderOss::Microsoft,
                    },
                    scheme_identifier,
                    client_id,
                    client_secret,
                    redirect_url,
                    scopes,
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
                    .api_security
                    .create(
                        &project.project_id.0,
                        &SecuritySchemeDataCloud {
                            provider_type: match provider_type {
                                IdentityProviderType::Google => ProviderCloud::Google,
                                IdentityProviderType::Facebook => ProviderCloud::Facebook,
                                IdentityProviderType::Gitlab => ProviderCloud::Gitlab,
                                IdentityProviderType::Microsoft => ProviderCloud::Microsoft,
                            },
                            scheme_identifier,
                            client_id,
                            client_secret,
                            redirect_url,
                            scopes,
                        },
                    )
                    .await
                    .map_service_error()?
                    .into()
            }
        };

        self.ctx.log_handler().log_view(&result);

        Ok(())
    }

    async fn cmd_get(
        &self,
        project: ProjectNameOptionalArg,
        security_scheme_id: String,
    ) -> anyhow::Result<()> {
        let project = self
            .ctx
            .cloud_project_handler()
            .opt_select_project(None /* TODO: account id */, project.project.as_ref())
            .await?;

        let result: ApiSecurityScheme = match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => clients
                .api_security
                .get(&security_scheme_id)
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
                    .api_security
                    .get(&project.project_id.0, &security_scheme_id)
                    .await
                    .map_service_error()?
                    .into()
            }
        };

        self.ctx.log_handler().log_view(&result);

        Ok(())
    }
}
