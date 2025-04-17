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

use crate::command::api::definition::ApiDefinitionSubcommand;
use crate::command::shared_args::ProjectNameOptionalArg;
use crate::command_handler::Handlers;
use crate::context::{Context, GolemClients};
use crate::error::service::AnyhowMapServiceError;
use crate::error::NonSuccessfulExit;
use crate::log::{log_action, log_warn_action, LogColorize, LogIndent};
use crate::model::api::{ApiDefinitionId, ApiDefinitionVersion};
use crate::model::app::{HttpApiDefinitionName, WithSource};
use crate::model::app_raw::{HttpApiDefinition, HttpApiDefinitionBindingType};
use crate::model::text::api_definition::{
    ApiDefinitionGetView, ApiDefinitionNewView, ApiDefinitionUpdateView,
};
use crate::model::text::fmt::log_error;
use crate::model::{PathBufOrStdin, ProjectNameAndId};
use anyhow::{bail, Context as AnyhowContext};
use golem_client::api::ApiDefinitionClient as ApiDefinitionClientOss;
use golem_client::model::{
    GatewayBindingComponent, GatewayBindingData, GatewayBindingType,
    HttpApiDefinitionRequest as HttpApiDefinitionRequestOss, HttpApiDefinitionRequest,
    HttpApiDefinitionResponseData, MethodPattern, RouteRequestData,
};
use golem_cloud_client::api::ApiDefinitionClient as ApiDefinitionClientCloud;
use golem_cloud_client::model::HttpApiDefinitionRequest as HttpApiDefinitionRequestCloud;
use serde::de::DeserializeOwned;
use std::sync::Arc;

pub struct ApiDefinitionCommandHandler {
    ctx: Arc<Context>,
}

impl ApiDefinitionCommandHandler {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    pub async fn handle_command(&mut self, command: ApiDefinitionSubcommand) -> anyhow::Result<()> {
        match command {
            ApiDefinitionSubcommand::New {
                project,
                definition,
            } => self.cmd_new(project, definition).await,
            ApiDefinitionSubcommand::Update {
                project,
                definition,
            } => self.cmd_update(project, definition).await,
            ApiDefinitionSubcommand::Import {
                project,
                definition,
            } => self.cmd_import(project, definition).await,
            ApiDefinitionSubcommand::Get {
                project,
                id,
                version,
            } => self.cmd_get(project, id, version).await,
            ApiDefinitionSubcommand::Delete {
                project,
                id,
                version,
            } => self.cmd_delete(project, id, version).await,
            ApiDefinitionSubcommand::List { project, id } => self.cmd_list(project, id).await,
        }
    }

    // TODO: drop
    async fn cmd_new(
        &self,
        project: ProjectNameOptionalArg,
        definition: PathBufOrStdin,
    ) -> anyhow::Result<()> {
        let project = self
            .ctx
            .cloud_project_handler()
            .opt_select_project(None /* TODO: account id */, project.project.as_ref())
            .await?;

        let result = match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => clients
                .api_definition
                .create_definition_json(&read_and_parse_api_definition(definition)?)
                .await
                .map_service_error()?,
            GolemClients::Cloud(clients) => {
                let project = self
                    .ctx
                    .cloud_project_handler()
                    .selected_project_or_default(project)
                    .await?;
                clients
                    .api_definition
                    .create_definition_json(
                        &project.project_id.0,
                        &read_and_parse_api_definition(definition)?,
                    )
                    .await
                    .map_service_error()?
            }
        };

        self.ctx
            .log_handler()
            .log_view(&ApiDefinitionNewView(result));

        Ok(())
    }

    async fn cmd_get(
        &self,
        project: ProjectNameOptionalArg,
        api_def_id: ApiDefinitionId,
        version: ApiDefinitionVersion,
    ) -> anyhow::Result<()> {
        let project = self
            .ctx
            .cloud_project_handler()
            .opt_select_project(None /* TODO: account id */, project.project.as_ref())
            .await?;

        match self
            .api_definition(project, &api_def_id.0, &version.0)
            .await?
        {
            Some(result) => {
                self.ctx
                    .log_handler()
                    .log_view(&ApiDefinitionGetView(result));
                Ok(())
            }
            None => {
                log_error("Not found");
                bail!(NonSuccessfulExit)
            }
        }
    }

    // TODO: drop
    async fn cmd_update(
        &self,
        project: ProjectNameOptionalArg,
        definition: PathBufOrStdin,
    ) -> anyhow::Result<()> {
        let project = self
            .ctx
            .cloud_project_handler()
            .opt_select_project(None /* TODO: account id */, project.project.as_ref())
            .await?;

        let result = match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => {
                let api_def: HttpApiDefinitionRequestOss =
                    read_and_parse_api_definition(definition)?;
                clients
                    .api_definition
                    .update_definition_json(&api_def.id, &api_def.version, &api_def)
                    .await
                    .map_service_error()?
            }
            GolemClients::Cloud(clients) => {
                let api_def: HttpApiDefinitionRequestCloud =
                    read_and_parse_api_definition(definition)?;
                let project = self
                    .ctx
                    .cloud_project_handler()
                    .selected_project_or_default(project)
                    .await?;
                clients
                    .api_definition
                    .update_definition_json(
                        &project.project_id.0,
                        &api_def.id,
                        &api_def.version,
                        &api_def,
                    )
                    .await
                    .map_service_error()?
            }
        };

        self.ctx
            .log_handler()
            .log_view(&ApiDefinitionUpdateView(result));

        Ok(())
    }

    // TODO: drop or make it a client side feature?
    async fn cmd_import(
        &self,
        project: ProjectNameOptionalArg,
        definition: PathBufOrStdin,
    ) -> anyhow::Result<()> {
        let project = self
            .ctx
            .cloud_project_handler()
            .opt_select_project(None /* TODO: account id */, project.project.as_ref())
            .await?;

        let result = match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => clients
                .api_definition
                .import_open_api_json(&read_and_parse_api_definition(definition)?)
                .await
                .map_service_error()?,
            GolemClients::Cloud(clients) => {
                let project = self
                    .ctx
                    .cloud_project_handler()
                    .selected_project_or_default(project)
                    .await?;
                clients
                    .api_definition
                    .import_open_api_json(
                        &project.project_id.0,
                        &read_and_parse_api_definition(definition)?,
                    )
                    .await
                    .map_service_error()?
            }
        };

        self.ctx
            .log_handler()
            .log_view(&ApiDefinitionUpdateView(result));

        Ok(())
    }

    async fn cmd_list(
        &self,
        project: ProjectNameOptionalArg,
        api_definition_id: Option<ApiDefinitionId>,
    ) -> anyhow::Result<()> {
        let project = self
            .ctx
            .cloud_project_handler()
            .opt_select_project(None /* TODO: account id */, project.project.as_ref())
            .await?;

        let definitions = match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => clients
                .api_definition
                .list_definitions(api_definition_id.as_ref().map(|id| id.0.as_str()))
                .await
                .map_service_error()?,
            GolemClients::Cloud(clients) => {
                let project = self
                    .ctx
                    .cloud_project_handler()
                    .selected_project_or_default(project)
                    .await?;
                clients
                    .api_definition
                    .list_definitions(
                        &project.project_id.0,
                        api_definition_id.as_ref().map(|id| id.0.as_str()),
                    )
                    .await
                    .map_service_error()?
            }
        };

        self.ctx.log_handler().log_view(&definitions);

        Ok(())
    }

    async fn cmd_delete(
        &self,
        project: ProjectNameOptionalArg,
        api_def_id: ApiDefinitionId,
        version: ApiDefinitionVersion,
    ) -> anyhow::Result<()> {
        let project = self
            .ctx
            .cloud_project_handler()
            .opt_select_project(None /* TODO: account id */, project.project.as_ref())
            .await?;

        match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => clients
                .api_definition
                .delete_definition(&api_def_id.0, &version.0)
                .await
                .map_service_error()?,
            GolemClients::Cloud(clients) => {
                let project = self
                    .ctx
                    .cloud_project_handler()
                    .selected_project_or_default(project)
                    .await?;
                clients
                    .api_definition
                    .delete_definition(&project.project_id.0, &api_def_id.0, &version.0)
                    .await
                    .map_service_error()?
            }
        };

        log_warn_action(
            "Deleted",
            format!(
                "API definition: {}/{}",
                api_def_id.0.log_color_highlight(),
                version.0.log_color_highlight()
            ),
        );

        Ok(())
    }

    pub async fn deploy(&self) -> anyhow::Result<()> {
        let api_definitions = {
            let app_ctx = self.ctx.app_context_lock().await;

            let app_ctx = app_ctx.some_or_err()?;

            // TODO: selection based on components
            app_ctx.application.api_definitions().clone()
        };

        if !api_definitions.is_empty() {
            log_action("Deploying", "HTTP API definitions");

            for (api_definition_name, api_definition) in api_definitions {
                let _indent = LogIndent::new();
                self.deploy_api_definition(&api_definition_name, &api_definition)
                    .await?;
            }
        }

        Ok(())
    }

    async fn deploy_api_definition(
        &self,
        api_definition_name: &HttpApiDefinitionName,
        api_definition: &WithSource<HttpApiDefinition>,
    ) -> anyhow::Result<()> {
        let manifest_api_definition =
            (api_definition_name, &api_definition.value).as_http_api_definition_request();

        log_action(
            "Deploying",
            format!(
                "HTTP API definition {}",
                api_definition_name.as_str().log_color_highlight()
            ),
        );

        // TODO: project
        let server_api_definition = self
            .api_definition(
                None,
                api_definition_name.as_str(),
                api_definition.value.version.as_str(),
            )
            .await?
            .map(|ad| ad.as_http_api_definition_request());

        match server_api_definition {
            Some(server_api_definition) => {
                if server_api_definition != manifest_api_definition {
                    todo!("diff")
                } else {
                    todo!("same")
                }
            }
            None => {
                let result = match self.ctx.golem_clients().await? {
                    GolemClients::Oss(clients) => clients
                        .api_definition
                        .create_definition_json(&manifest_api_definition)
                        .await
                        .map_service_error()?,
                    GolemClients::Cloud(clients) => {
                        let project = self
                            .ctx
                            .cloud_project_handler()
                            .selected_project_or_default(None) // TODO: to the top of deploy
                            .await?;

                        clients
                            .api_definition
                            .create_definition_json(
                                &project.project_id.0,
                                // TODO: would be nice to share the model between oss and cloud instead of "re-encoding"
                                &parse_api_definition(&serde_yaml::to_string(
                                    &manifest_api_definition,
                                )?)?,
                            )
                            .await
                            .map_service_error()?
                    }
                };

                self.ctx
                    .log_handler()
                    .log_view(&ApiDefinitionNewView(result));

                Ok(())
            }
        }
    }

    async fn api_definition(
        &self,
        project: Option<ProjectNameAndId>, // TODO: ref?
        name: &str,
        version: &str,
    ) -> anyhow::Result<Option<HttpApiDefinitionResponseData>> {
        match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => clients
                .api_definition
                .get_definition(name, version)
                .await
                .map_service_error_not_found_as_opt(),
            GolemClients::Cloud(clients) => {
                let project = self
                    .ctx
                    .cloud_project_handler()
                    .selected_project_or_default(project)
                    .await?;
                clients
                    .api_definition
                    .get_definition(&project.project_id.0, name, version)
                    .await
                    .map_service_error_not_found_as_opt()
            }
        }
    }
}

fn parse_api_definition<T: DeserializeOwned>(input: &str) -> anyhow::Result<T> {
    serde_yaml::from_str(input).context("Failed to parse API definition")
}

fn read_and_parse_api_definition<T: DeserializeOwned>(source: PathBufOrStdin) -> anyhow::Result<T> {
    parse_api_definition(&source.read_to_string()?)
}

trait AsHttpApiDefinitionRequest {
    fn as_http_api_definition_request(&self) -> HttpApiDefinitionRequest;
}

impl AsHttpApiDefinitionRequest for HttpApiDefinitionResponseData {
    fn as_http_api_definition_request(&self) -> HttpApiDefinitionRequest {
        HttpApiDefinitionRequest {
            id: self.id.clone(),
            version: self.version.clone(),
            security: None, // TODO: check that this is not needed anymore
            routes: self
                .routes
                .iter()
                .map(|route| RouteRequestData {
                    method: route.method.clone(),
                    path: route.path.clone(),
                    binding: GatewayBindingData {
                        binding_type: route.binding.binding_type.clone(),
                        component: route.binding.component.as_ref().map(|component| {
                            GatewayBindingComponent {
                                name: component.name.clone(),
                                version: None, // TODO: None for now, how to handle diff on this?
                            }
                        }),
                        worker_name: route.binding.worker_name.clone(),
                        idempotency_key: route.binding.idempotency_key.clone(),
                        response: route.binding.response.clone(),
                        invocation_context: None, // TODO: should this be in the response?
                        allow_origin: None,       // TODO: check that this is not needed anymore
                        allow_methods: None,      // TODO: check that this is not needed anymore
                        allow_headers: None,      // TODO: check that this is not needed anymore
                        expose_headers: None,     // TODO: check that this is not needed anymore
                        max_age: None,            // TODO: check that this is not needed anymore
                        allow_credentials: None,  // TODO: check that this is not needed anymore
                    },
                    cors: None, // TODO: handle cors
                    security: route.security.clone(),
                })
                .collect(),
            draft: self.draft,
        }
    }
}

// TODO: wrapper for the tuple (especially once CORS representation is finalised)
impl AsHttpApiDefinitionRequest for (&HttpApiDefinitionName, &HttpApiDefinition) {
    fn as_http_api_definition_request(&self) -> HttpApiDefinitionRequest {
        let (name, api_definition) = self;

        HttpApiDefinitionRequest {
            id: name.to_string(),
            version: api_definition.version.clone(),
            security: None, // TODO: check that this is not needed anymore
            routes: api_definition
                .routes
                .iter()
                .map(|route| RouteRequestData {
                    method: to_method_pattern(&route.method),
                    path: route.path.clone(),
                    binding: GatewayBindingData {
                        binding_type: route.binding.type_.as_ref().map(|binding_type| {
                            match binding_type {
                                HttpApiDefinitionBindingType::Default => {
                                    GatewayBindingType::Default
                                }
                                HttpApiDefinitionBindingType::FileServer => {
                                    GatewayBindingType::FileServer
                                }
                                HttpApiDefinitionBindingType::HttpHandler => {
                                    GatewayBindingType::HttpHandler
                                }
                            }
                        }),
                        component: Some(GatewayBindingComponent {
                            name: route.binding.component_name.clone(),
                            version: None, // TODO: how we should handle versions
                        }),
                        worker_name: route.binding.worker_name.clone(),
                        idempotency_key: route.binding.idempotency_key.clone(),
                        response: route.binding.response.clone(),
                        invocation_context: None, // TODO: should this be in the response?
                        allow_origin: None,       // TODO: check that this is not needed anymore
                        allow_methods: None,      // TODO: check that this is not needed anymore
                        allow_headers: None,      // TODO: check that this is not needed anymore
                        expose_headers: None,     // TODO: check that this is not needed anymore
                        max_age: None,            // TODO: check that this is not needed anymore
                        allow_credentials: None,  // TODO: check that this is not needed anymore
                    },
                    cors: None, // TODO:
                    security: route.security.clone(),
                })
                .collect(),
            draft: false,
        }
    }
}

// TODO: add validation for this in the manifest
fn to_method_pattern(method: &str) -> MethodPattern {
    match method.to_lowercase().as_str() {
        "get" => MethodPattern::Get,
        "connect" => MethodPattern::Connect,
        "post" => MethodPattern::Post,
        "delete" => MethodPattern::Delete,
        "put" => MethodPattern::Put,
        "patch" => MethodPattern::Patch,
        "options" => MethodPattern::Options,
        "trace" => MethodPattern::Trace,
        "head" => MethodPattern::Head,
        _ => unreachable!(), // TODO
    }
}
