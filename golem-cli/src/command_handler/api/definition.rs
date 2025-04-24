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
use crate::log::{log_warn_action, LogColorize};
use crate::model::text::api_definition::{
    ApiDefinitionExportView, ApiDefinitionGetView, ApiDefinitionNewView, ApiDefinitionUpdateView,
};
use crate::model::{
    ApiDefinitionId, ApiDefinitionVersion, OpenApiDefinitionOutputFormat, PathBufOrStdin,
};
use anyhow::Context as AnyhowContext;
use golem_client::api::ApiDefinitionClient as ApiDefinitionClientOss;
use golem_client::model::HttpApiDefinitionRequest as HttpApiDefinitionRequestOss;
use golem_cloud_client::api::ApiDefinitionClient as ApiDefinitionClientCloud;
use golem_cloud_client::model::HttpApiDefinitionRequest as HttpApiDefinitionRequestCloud;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use webbrowser;
use std::net::TcpListener;
use warp;

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
            ApiDefinitionSubcommand::List { project, id } => self.list(project, id).await,
            ApiDefinitionSubcommand::Export {
                project,
                id,
                version,
                format,
                output_name,
            } => {
                self.cmd_export(project, id, version, format, output_name)
                    .await
            }
            ApiDefinitionSubcommand::Swagger {
                project,
                id,
                version,
                host,
            } => self.cmd_swagger(project, id, version, host).await,
        }
    }

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

        let result = match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => clients
                .api_definition
                .get_definition(&api_def_id.0, &version.0)
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
                    .get_definition(&project.project_id.0, &api_def_id.0, &version.0)
                    .await
                    .map_service_error()?
            }
        };

        self.ctx
            .log_handler()
            .log_view(&ApiDefinitionGetView(result));

        Ok(())
    }

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

    async fn list(
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

    async fn cmd_export(
        &self,
        project: ProjectNameOptionalArg,
        id: ApiDefinitionId,
        version: ApiDefinitionVersion,
        format: OpenApiDefinitionOutputFormat,
        output_name: Option<String>,
    ) -> anyhow::Result<()> {
        let _project = self
            .ctx
            .cloud_project_handler()
            .opt_select_project(None, project.project.as_ref())
            .await?;

        let result = match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => {
                let response = clients
                    .api_definition
                    .export_definition(&id.0, &version.0)
                    .await
                    .map_service_error()?;

                let openapi_spec = response.openapi_yaml;
                let file_name = output_name.unwrap_or_else(|| format!("{}_{}", id.0, version.0));
                let file_path = match format {
                    OpenApiDefinitionOutputFormat::Json => format!("{}.json", file_name),
                    OpenApiDefinitionOutputFormat::Yaml => format!("{}.yaml", file_name),
                };

                match format {
                    OpenApiDefinitionOutputFormat::Json => {
                        // Convert YAML to JSON for the JSON format option
                        let yaml_obj: serde_yaml::Value = serde_yaml::from_str(&openapi_spec)?;
                        let json_obj: serde_json::Value = serde_json::to_value(yaml_obj)?;
                        let json_str = serde_json::to_string_pretty(&json_obj)?;
                        std::fs::write(&file_path, json_str)?;
                    }
                    OpenApiDefinitionOutputFormat::Yaml => {
                        // Use YAML directly for the YAML format option
                        std::fs::write(&file_path, openapi_spec)?;
                    }
                }

                format!("Exported to {}", file_path)
            }
            GolemClients::Cloud(_) => {
                // Export is not supported in Golem Cloud
                "API definition export is not supported in Golem Cloud".to_string()
            }
        };

        self.ctx
            .log_handler()
            .log_view(&ApiDefinitionExportView(result));

        Ok(())
    }

    async fn cmd_swagger(
        &self,
        project: ProjectNameOptionalArg,
        id: ApiDefinitionId,
        version: ApiDefinitionVersion,
        host: String,
    ) -> anyhow::Result<()> {
        let _project = self
            .ctx
            .cloud_project_handler()
            .opt_select_project(None, project.project.as_ref())
            .await?;

        // Check if using Golem Cloud, which doesn't support export
        if matches!(self.ctx.golem_clients().await?, GolemClients::Cloud(_)) {
            self.ctx.log_handler().log_view(
                &ApiDefinitionExportView("Swagger UI is not supported in Golem Cloud as API definition export is not available".to_string())
            );
            return Ok(());
        }

        // First export the API spec
        let openapi_spec = match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => {
                let response = clients
                    .api_definition
                    .export_definition(&id.0, &version.0)
                    .await
                    .map_service_error()?;
                response.openapi_yaml
            }
            GolemClients::Cloud(_) => {
                // This code is unreachable now with the check above, but keeping it for safety
                return Ok(());
            }
        };

        // Parse the YAML spec into JSON Value
        let mut spec: serde_json::Value = serde_yaml::from_str(&openapi_spec)?;

        // Filter paths to only include methods with binding-type: default
        filter_routes(&mut spec);

        // Fetch deployments
        let deployments = match self.ctx.golem_clients().await? {
            GolemClients::Oss(clients) => {
                use golem_client::api::ApiDeploymentClient;
                clients
                    .api_deployment
                    .list_deployments(Some(&id.0))
                    .await
                    .map_service_error()?
            }
            GolemClients::Cloud(_) => {
                // This code is unreachable now with the check above, but keeping it for safety
                return Ok(());
            }
        };

        // Add server information if deployments exist
        if !deployments.is_empty() {
            // Initialize servers array if it doesn't exist
            if !spec.as_object().unwrap().contains_key("servers") {
                spec.as_object_mut()
                    .unwrap()
                    .insert("servers".to_string(), serde_json::Value::Array(Vec::new()));
            }

            // Add servers to the spec
            if let Some(servers) = spec.get_mut("servers") {
                if let Some(servers) = servers.as_array_mut() {
                    // Add deployment servers with HTTP
                    for deployment in &deployments {
                        let url = match &deployment.site.subdomain {
                            Some(subdomain) => {
                                format!("http://{}.{}", subdomain, deployment.site.host)
                            }
                            None => format!("http://{}", deployment.site.host),
                        };
                        servers.push(serde_json::json!({
                            "url": url,
                            "description": "Deployed instance"
                        }));
                    }
                }
            }
        }

        // Handle localhost case
        if host.starts_with("localhost") || host.starts_with("127.0.0.1") {
            let port = host
                .split(':')
                .nth(1)
                .unwrap_or("9990")
                .parse::<u16>()
                .map_err(|_| anyhow::anyhow!("Invalid port number"))?;

            // Start local Swagger UI server
            self.start_local_swagger_ui(serde_yaml::to_string(&spec)?, port)
                .await?;

            self.ctx
                .log_handler()
                .log_view(&ApiDefinitionExportView(format!(
                    "Swagger UI running at http://localhost:{}\n{}",
                    port,
                    if deployments.is_empty() {
                        "No deployments found - displaying API schema without server information"
                            .to_string()
                    } else {
                        format!("API is deployed at {} locations", deployments.len())
                    }
                )));

            // Wait for ctrl+c
            tokio::signal::ctrl_c().await?;

            return Ok(());
        }

        // For non-localhost cases, just save the file
        std::fs::write("openapi.yaml", serde_yaml::to_string(&spec)?)?;

        self.ctx
            .log_handler()
            .log_view(&ApiDefinitionExportView(format!(
                "OpenAPI spec saved to openapi.yaml\n{}",
                if deployments.is_empty() {
                    "No deployments found - displaying API schema without server information"
                        .to_string()
                } else {
                    format!("API is deployed at {} locations", deployments.len())
                }
            )));

        Ok(())
    }

    async fn start_local_swagger_ui(&self, spec: String, port: u16) -> anyhow::Result<()> {
        use std::net::TcpListener;
        use std::sync::Arc;
        use warp::Filter;

        // First check if the port is available
        let listener = TcpListener::bind(("127.0.0.1", port));
        if listener.is_err() {
            return Err(anyhow::anyhow!(
                "Port {} is already in use. Please choose a different port.",
                port
            ));
        }
        drop(listener); // Close the test connection

        // Parse the YAML to ensure it's valid
        let spec_value: serde_yaml::Value = serde_yaml::from_str(&spec)?;

        // Convert to JSON for Swagger UI
        let spec_json = serde_json::to_value(&spec_value)?;

        // Create a shared spec for the server
        let spec = Arc::new(spec_json);

        // Serve the OpenAPI spec as JSON
        let openapi_route =
            warp::path("openapi.json").map(move || warp::reply::json(spec.clone().as_ref()));

        // Serve the Swagger UI HTML
        let swagger_ui_html = r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <title>Swagger UI</title>
                <link rel="stylesheet" type="text/css" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css">
                <script src="https://cdn.jsdelivr.net/npm/swagger-ui_dist@5/swagger-ui-bundle.js"></script>
            </head>
            <body>
                <div id="swagger-ui"></div>
                <script>
                    window.onload = () => {
                        window.ui = SwaggerUIBundle({
                            url: '/openapi.json',
                            dom_id: '#swagger-ui',
                            deepLinking: true,
                            presets: [
                                SwaggerUIBundle.presets.apis,
                                SwaggerUIBundle.SwaggerUIStandalonePreset
                            ],
                        });
                    };
                </script>
            </body>
            </html>
        "#;

        let swagger_route = warp::path::end().map(move || warp::reply::html(swagger_ui_html));

        // Combine routes
        let routes = openapi_route.or(swagger_route);

        // Start server in a background task
        tokio::spawn(async move {
            warp::serve(routes).run(([127, 0, 0, 1], port)).await;
        });

        // Try to open browser
        match webbrowser::open(&format!("http://localhost:{}", port)) {
            Ok(_) => println!("Browser opened successfully."),
            Err(_) => println!(
                "Could not open browser automatically. You can access the Swagger UI at: http://localhost:{}",
                port
            ),
        }

        Ok(())
    }
}

/// Filters the OpenAPI specification to only include methods with `binding-type: default`.
fn filter_routes(spec: &mut serde_json::Value) {
    if let Some(paths) = spec.get_mut("paths").and_then(|p| p.as_object_mut()) {
        let paths_to_remove: Vec<String> = paths
            .iter_mut()
            .filter_map(|(path, methods)| {
                if let Some(methods) = methods.as_object_mut() {
                    // Filter methods within this path
                    let methods_to_remove: Vec<String> = methods
                        .iter()
                        .filter_map(|(method, details)| {
                            // Check if the method has a binding-type and if it's not "default"
                            if let Some(binding) = details.get("x-golem-api-gateway-binding") {
                                if let Some(binding_type) = binding.get("binding-type") {
                                    if binding_type != "default" {
                                        return Some(method.clone());
                                    }
                                }
                            }
                            None
                        })
                        .collect();

                    // Remove filtered methods
                    for method in methods_to_remove {
                        methods.remove(&method);
                    }

                    // If no methods left in this path, mark it for removal
                    if methods.is_empty() {
                        return Some(path.clone());
                    }
                }
                None
            })
            .collect();

        // Remove empty paths
        for path in paths_to_remove {
            paths.remove(&path);
        }
    }
}

fn parse_api_definition<T: DeserializeOwned>(input: &str) -> anyhow::Result<T> {
    serde_yaml::from_str(input).context("Failed to parse API definition")
}

fn read_and_parse_api_definition<T: DeserializeOwned>(source: PathBufOrStdin) -> anyhow::Result<T> {
    parse_api_definition(&source.read_to_string()?)
}
