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

use crate::app::yaml_edit::AppYamlEditor;
use crate::command::api::definition::ApiDefinitionSubcommand;
use crate::command::shared_args::{ProjectNameOptionalArg, WorkerUpdateOrRedeployArgs};
use crate::command_handler::Handlers;
use crate::context::{Context, GolemClients};
use crate::error::service::AnyhowMapServiceError;
use crate::error::NonSuccessfulExit;
use crate::log::{
    log_action, log_skipping_up_to_date, log_warn_action, logln, LogColorize, LogIndent,
};
use crate::model::api::{ApiDefinitionId, ApiDefinitionVersion, HttpApiDeployMode};
use crate::model::app::{ApplicationComponentSelectMode, HttpApiDefinitionName, WithSource};
use crate::model::app_raw::HttpApiDefinition;
use crate::model::component::Component;
use crate::model::deploy_diff::{
    AsHttpApiDefinitionRequest, HttpApiDefinitionDeployableManifestSource, ToYamlValueWithoutNulls,
};
use crate::model::text::api_definition::{
    ApiDefinitionGetView, ApiDefinitionNewView, ApiDefinitionUpdateView,
};
use crate::model::text::fmt::{log_deployable_entity_yaml_diff, log_error, log_warn};
use crate::model::{ComponentName, PathBufOrStdin, ProjectNameAndId};
use anyhow::{bail, Context as AnyhowContext};
use golem_client::api::ApiDefinitionClient as ApiDefinitionClientOss;
use golem_client::model::{HttpApiDefinitionRequest, HttpApiDefinitionResponseData};
use golem_cloud_client::api::ApiDefinitionClient as ApiDefinitionClientCloud;
use itertools::Itertools;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct ApiDefinitionCommandHandler {
    ctx: Arc<Context>,
}

impl ApiDefinitionCommandHandler {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    pub async fn handle_command(&self, command: ApiDefinitionSubcommand) -> anyhow::Result<()> {
        match command {
            ApiDefinitionSubcommand::Deploy {
                http_api_definition_name,
            } => self.cmd_deploy(http_api_definition_name).await,
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

    async fn cmd_deploy(&self, name: Option<HttpApiDefinitionName>) -> anyhow::Result<()> {
        let project = None::<ProjectNameAndId>; // TODO

        let used_component_names = {
            {
                let app_ctx = self.ctx.app_context_lock().await;
                let app_ctx = app_ctx.some_or_err()?;
                match name.as_ref() {
                    Some(name) => {
                        if !app_ctx
                            .application
                            .http_api_definitions()
                            .keys()
                            .contains(name)
                        {
                            log_error(format!(
                                "HTTP API definition {} not found in the application manifest",
                                name.as_str().log_color_highlight()
                            ));
                            logln("");
                            bail!(NonSuccessfulExit)
                            // TODO: show available API names
                        }

                        app_ctx
                            .application
                            .used_component_names_for_http_api_definition(name)
                    }
                    None => app_ctx
                        .application
                        .used_component_names_for_all_http_api_definition(),
                }
            }
            .into_iter()
            .map(|component_name| ComponentName::from(component_name.to_string()))
            .collect::<Vec<_>>()
        };

        let components = {
            if !used_component_names.is_empty() {
                self.ctx
                    .component_handler()
                    .deploy(
                        project.as_ref(),
                        used_component_names,
                        None,
                        &ApplicationComponentSelectMode::All,
                        WorkerUpdateOrRedeployArgs::default(),
                    )
                    .await?
                    .into_iter()
                    .map(|component| (component.component_name.0.clone(), component))
                    .collect::<BTreeMap<_, _>>()
            } else {
                BTreeMap::new()
            }
        };

        match &name {
            Some(name) => {
                let definition = {
                    let app_ctx = self.ctx.app_context_lock().await;
                    let app_ctx = app_ctx.some_or_err()?;
                    app_ctx
                        .application
                        .http_api_definitions()
                        .get(name)
                        .unwrap()
                        .clone()
                };

                self.deploy_api_definition(
                    project.as_ref(),
                    HttpApiDeployMode::All,
                    &components,
                    name,
                    &definition,
                )
                .await
            }
            None => {
                self.deploy(project.as_ref(), HttpApiDeployMode::All, &components)
                    .await
            }
        }
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
            .api_definition(project.as_ref(), &api_def_id.0, &version.0)
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
            GolemClients::Cloud(clients) => clients
                .api_definition
                .import_open_api_json(
                    &self
                        .ctx
                        .cloud_project_handler()
                        .selected_project_id_or_default(project.as_ref())
                        .await?
                        .0,
                    &read_and_parse_api_definition(definition)?,
                )
                .await
                .map_service_error()?,
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
            GolemClients::Cloud(clients) => clients
                .api_definition
                .list_definitions(
                    &self
                        .ctx
                        .cloud_project_handler()
                        .selected_project_id_or_default(project.as_ref())
                        .await?
                        .0,
                    api_definition_id.as_ref().map(|id| id.0.as_str()),
                )
                .await
                .map_service_error()?,
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
            GolemClients::Cloud(clients) => clients
                .api_definition
                .delete_definition(
                    &self
                        .ctx
                        .cloud_project_handler()
                        .selected_project_id_or_default(project.as_ref())
                        .await?
                        .0,
                    &api_def_id.0,
                    &version.0,
                )
                .await
                .map_service_error()?,
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

    pub async fn deploy(
        &self,
        project: Option<&ProjectNameAndId>,
        deploy_mode: HttpApiDeployMode,
        latest_component_versions: &BTreeMap<String, Component>,
    ) -> anyhow::Result<()> {
        let api_definitions = {
            let app_ctx = self.ctx.app_context_lock().await;
            let app_ctx = app_ctx.some_or_err()?;
            app_ctx.application.http_api_definitions().clone()
        };

        if !api_definitions.is_empty() {
            log_action("Deploying", "HTTP API definitions");

            for (api_definition_name, api_definition) in api_definitions {
                let _indent = LogIndent::new();
                self.deploy_api_definition(
                    project,
                    deploy_mode,
                    latest_component_versions,
                    &api_definition_name,
                    &api_definition,
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn deploy_api_definition(
        &self,
        project: Option<&ProjectNameAndId>,
        deploy_mode: HttpApiDeployMode,
        latest_component_versions: &BTreeMap<String, Component>,
        api_definition_name: &HttpApiDefinitionName,
        api_definition: &WithSource<HttpApiDefinition>,
    ) -> anyhow::Result<()> {
        let skip_by_component_filter = match deploy_mode {
            HttpApiDeployMode::All => false,
            HttpApiDeployMode::Matching => !api_definition.value.routes.iter().any(|route| {
                match &route.binding.component_name {
                    Some(component_name) => latest_component_versions.contains_key(component_name),
                    None => false,
                }
            }),
        };

        if skip_by_component_filter {
            log_warn_action(
                "Skipping",
                format!(
                    "deploying HTTP API definition {}, not matched by component selection",
                    api_definition_name.as_str().log_color_highlight()
                ),
            );
            return Ok(());
        };

        let server_api_definition = self
            .api_definition(
                project,
                api_definition_name.as_str(),
                api_definition.value.version.as_str(),
            )
            .await?
            .map(|ad| ad.as_http_api_definition_request())
            .transpose()?;

        let manifest_api_definition = {
            let mut manifest_api_definition = HttpApiDefinitionDeployableManifestSource {
                name: api_definition_name,
                api_definition: &api_definition.value,
                latest_component_versions,
            }
            .as_http_api_definition_request()?;

            // NOTE: if the only diff if being non-draft on serverside, we hide that
            if let Some(server_api_definition) = &server_api_definition {
                if manifest_api_definition.version == server_api_definition.version
                    && !server_api_definition.draft
                    && manifest_api_definition.draft
                {
                    manifest_api_definition.draft = false;
                }
            }

            manifest_api_definition
        };

        let manifest_api_definition_yaml = manifest_api_definition
            .clone()
            .to_yaml_value_without_nulls()?;

        let server_api_definition_yaml = server_api_definition
            .clone()
            .map(|ad| ad.to_yaml_value_without_nulls())
            .transpose()?;

        match server_api_definition_yaml {
            Some(server_api_definition_yaml) => {
                if server_api_definition_yaml != manifest_api_definition_yaml {
                    log_warn_action(
                        "Found",
                        format!(
                            "changes in HTTP API definition {}@{}",
                            api_definition_name.as_str().log_color_highlight(),
                            manifest_api_definition
                                .version
                                .as_str()
                                .log_color_highlight()
                        ),
                    );

                    {
                        let _indent = self.ctx.log_handler().nested_text_view_indent();
                        log_deployable_entity_yaml_diff(
                            &server_api_definition_yaml,
                            &manifest_api_definition_yaml,
                        )?;
                    }

                    if server_api_definition.map(|ad| ad.draft) == Some(true) {
                        log_action(
                            "Updating",
                            format!(
                                "HTTP API definition {}",
                                api_definition_name.as_str().log_color_highlight()
                            ),
                        );

                        let result = self
                            .update_api_definition(project, &manifest_api_definition)
                            .await?;

                        self.ctx
                            .log_handler()
                            .log_view(&ApiDefinitionUpdateView(result));

                        Ok(())
                    } else {
                        log_warn(
                            "The current version of the HTTP API is already deployed as non-draft.",
                        );

                        match self
                            .ctx
                            .interactive_handler()
                            .select_new_api_definition_version(&manifest_api_definition)?
                        {
                            Some(new_version) => {
                                let new_draft = true;
                                let old_version = manifest_api_definition.version.clone();

                                let manifest_api_definition = {
                                    let mut manifest_api_definition = manifest_api_definition;
                                    manifest_api_definition.version = new_version;
                                    manifest_api_definition.draft = new_draft;
                                    manifest_api_definition
                                };

                                {
                                    let app_ctx = self.ctx.app_context_lock().await;
                                    let app_ctx = app_ctx.some_or_err()?;

                                    let mut editor = AppYamlEditor::new(&app_ctx.application);
                                    editor.update_api_definition_version(
                                        api_definition_name,
                                        &manifest_api_definition.version,
                                        new_draft,
                                    )?;
                                    editor.update_documents()?;
                                }

                                log_action(
                                    "Creating",
                                    format!(
                                        "new HTTP API definition version for {}, with version updated from {} to {}",
                                        api_definition_name.as_str().log_color_highlight(),
                                        old_version.log_color_highlight(),
                                        manifest_api_definition
                                            .version
                                            .as_str()
                                            .log_color_highlight()
                                    ),
                                );

                                let result = self
                                    .new_api_definition(project, &manifest_api_definition)
                                    .await?;

                                self.ctx
                                    .log_handler()
                                    .log_view(&ApiDefinitionNewView(result));

                                Ok(())
                            }
                            None => {
                                log_error(format!(
                                    "Please specify a new version for {} in {}",
                                    api_definition_name.as_str().log_color_highlight(),
                                    api_definition.source.log_color_highlight()
                                ));
                                bail!(NonSuccessfulExit)
                            }
                        }
                    }
                } else {
                    log_skipping_up_to_date(format!(
                        "deploying HTTP API definition {}",
                        api_definition_name.as_str().log_color_highlight()
                    ));
                    Ok(())
                }
            }
            None => {
                log_action(
                    "Creating",
                    format!(
                        "new HTTP API definition version {}@{}",
                        api_definition_name.as_str().log_color_highlight(),
                        manifest_api_definition
                            .version
                            .as_str()
                            .log_color_highlight()
                    ),
                );

                let result = self
                    .new_api_definition(project, &manifest_api_definition)
                    .await?;

                self.ctx
                    .log_handler()
                    .log_view(&ApiDefinitionNewView(result));

                Ok(())
            }
        }
    }

    async fn api_definition(
        &self,
        project: Option<&ProjectNameAndId>,
        name: &str,
        version: &str,
    ) -> anyhow::Result<Option<HttpApiDefinitionResponseData>> {
        match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => clients
                .api_definition
                .get_definition(name, version)
                .await
                .map_service_error_not_found_as_opt(),
            GolemClients::Cloud(clients) => clients
                .api_definition
                .get_definition(
                    &self
                        .ctx
                        .cloud_project_handler()
                        .selected_project_id_or_default(project)
                        .await?
                        .0,
                    name,
                    version,
                )
                .await
                .map_service_error_not_found_as_opt(),
        }
    }

    async fn update_api_definition(
        &self,
        project: Option<&ProjectNameAndId>,
        manifest_api_definition: &HttpApiDefinitionRequest,
    ) -> anyhow::Result<HttpApiDefinitionResponseData> {
        match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => clients
                .api_definition
                .update_definition_json(
                    &manifest_api_definition.id,
                    &manifest_api_definition.version,
                    manifest_api_definition,
                )
                .await
                .map_service_error(),
            GolemClients::Cloud(clients) => {
                clients
                    .api_definition
                    .update_definition_json(
                        &self
                            .ctx
                            .cloud_project_handler()
                            .selected_project_id_or_default(project)
                            .await?
                            .0,
                        &manifest_api_definition.id,
                        &manifest_api_definition.version,
                        // TODO: would be nice to share the model between oss and cloud instead of "re-encoding"
                        &parse_api_definition(&serde_yaml::to_string(&manifest_api_definition)?)?,
                    )
                    .await
                    .map_service_error()
            }
        }
    }

    async fn new_api_definition(
        &self,
        project: Option<&ProjectNameAndId>,
        api_definition: &HttpApiDefinitionRequest,
    ) -> anyhow::Result<HttpApiDefinitionResponseData> {
        match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => clients
                .api_definition
                .create_definition_json(api_definition)
                .await
                .map_service_error(),
            GolemClients::Cloud(clients) => {
                clients
                    .api_definition
                    .create_definition_json(
                        &self
                            .ctx
                            .cloud_project_handler()
                            .selected_project_id_or_default(project)
                            .await?
                            .0,
                        // TODO: would be nice to share the model between oss and cloud instead of "re-encoding"
                        &parse_api_definition(&serde_yaml::to_string(&api_definition)?)?,
                    )
                    .await
                    .map_service_error()
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
