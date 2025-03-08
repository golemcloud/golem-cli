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

use crate::command::component::ComponentSubcommand;
use crate::command::shared_args::{BuildArgs, ForceBuildArg};
use crate::command_handler::app::AppCommandHandler;
use crate::command_handler::{to_service_error, CommandHandler};
use crate::context::GolemClients;
use crate::model::component::Component;
use crate::model::text::component::{ComponentCreateView, ComponentUpdateView};
use crate::model::ComponentName;
use anyhow::{anyhow, Context};
use golem_client::api::ComponentClient as ComponentClientOss;
use golem_wasm_rpc_stubgen::commands::app::ComponentSelectMode;
use golem_wasm_rpc_stubgen::log::{log_action, LogColorize, LogIndent};
use tokio::fs::File;
use tracing::debug;
use uuid::Uuid;

pub trait ComponentCommandHandler {
    fn base(&self) -> &CommandHandler;
    fn base_mut(&mut self) -> &mut CommandHandler;

    async fn handle_component_subcommand(
        &mut self,
        subcommand: ComponentSubcommand,
    ) -> anyhow::Result<()> {
        match subcommand {
            ComponentSubcommand::New { .. } => {
                todo!()
            }
            ComponentSubcommand::Build {
                component_name,
                build: build_args,
            } => {
                self.base_mut()
                    .build(
                        component_name.component_name,
                        Some(build_args),
                        &ComponentSelectMode::CurrentDir,
                    )
                    .await
            }
            ComponentSubcommand::Deploy {
                component_name,
                force_build,
            } => {
                self.base_mut()
                    .deploy(
                        component_name.component_name,
                        Some(force_build),
                        &ComponentSelectMode::CurrentDir,
                    )
                    .await
            }
            ComponentSubcommand::Clean { component_name } => {
                self.base_mut()
                    .clean(
                        component_name.component_name,
                        &ComponentSelectMode::CurrentDir,
                    )
                    .await
            }
        }
    }

    async fn deploy(
        &mut self,
        component_names: Vec<ComponentName>,
        force_build: Option<ForceBuildArg>,
        default_component_select_mode: &ComponentSelectMode,
    ) -> anyhow::Result<()> {
        self.base_mut()
            .build(
                component_names,
                force_build.map(|force_build| BuildArgs {
                    step: vec![],
                    force_build,
                }),
                default_component_select_mode,
            )
            .await?;

        // TODO: hash <-> version check for skipping deploy

        let selected_component_names = self
            .base()
            .required_application_context()
            .await?
            .selected_component_names()
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        log_action("Deploying", "components");

        for component_name in &selected_component_names {
            let _indent = LogIndent::new();

            let component_id = self.component_id_by_name(component_name.as_str()).await?;
            let app_ctx = self.base().required_application_context().await?;
            let component_linked_wasm_path = app_ctx
                .application
                .component_linked_wasm(component_name, self.base().ctx.build_profile());
            let component_linked_wasm = File::open(&component_linked_wasm_path)
                .await
                .with_context(|| {
                    anyhow!(
                        "Failed to open component linked WASM at {}",
                        component_linked_wasm_path
                            .display()
                            .to_string()
                            .log_color_error_highlight()
                    )
                })?;

            let component_properties = &app_ctx
                .application
                .component_properties(component_name, self.base().ctx.build_profile())
                .clone();
            let component_extensions = &component_properties.extensions;
            let component_dynamic_linking = self
                .base_mut()
                .app_component_dynamic_linking_oss(component_name)
                .await?;

            match &component_id {
                Some(component_id) => {
                    log_action(
                        "Updating",
                        format!(
                            "component {}",
                            component_name.as_str().log_color_highlight()
                        ),
                    );
                    let _indent = CommandHandler::nested_text_view_indent();
                    match self.base().ctx.golem_clients().await? {
                        GolemClients::Oss(clients) => {
                            let component = clients
                                .component
                                .update_component(
                                    component_id,
                                    Some(&component_extensions.component_type),
                                    component_linked_wasm,
                                    None,         // TODO:
                                    None::<File>, // TODO:
                                    component_dynamic_linking.as_ref(),
                                )
                                .await
                                .map_err(to_service_error)?;
                            self.base()
                                .log_view(&ComponentUpdateView(Component::from(component).into()));
                        }
                        GolemClients::Cloud(_) => {
                            todo!()
                        }
                    }
                }
                None => {
                    log_action(
                        "Creating",
                        format!(
                            "component {}",
                            component_name.as_str().log_color_highlight()
                        ),
                    );
                    let _indent = CommandHandler::nested_text_view_indent();
                    match self.base().ctx.golem_clients().await? {
                        GolemClients::Oss(clients) => {
                            let component = clients
                                .component
                                .create_component(
                                    component_name.as_str(),
                                    Some(&component_extensions.component_type),
                                    component_linked_wasm,
                                    None,         // TODO:
                                    None::<File>, // TODO:
                                    component_dynamic_linking.as_ref(),
                                )
                                .await
                                .map_err(to_service_error)?;
                            self.base()
                                .log_view(&ComponentCreateView(Component::from(component).into()));
                        }
                        GolemClients::Cloud(_) => {
                            todo!()
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // TODO: we might want to have a filter for batch name lookups on the server side
    // TODO: also the search returns all versions
    // TODO: maybe add transient or persistent cache for all the meta
    async fn service_component_by_name(
        &self,
        component_name: &str,
    ) -> anyhow::Result<Option<Component>> {
        match self.base().ctx.golem_clients().await? {
            GolemClients::Oss(clients) => {
                let mut components = clients
                    .component
                    .get_components(Some(component_name))
                    .await
                    .map_err(to_service_error)?;
                debug!(components = ?components, "service_component_by_name");
                if !components.is_empty() {
                    Ok(Some(Component::from(components.pop().unwrap())))
                } else {
                    Ok(None)
                }
            }
            GolemClients::Cloud(_) => {
                todo!()
            }
        }
    }

    async fn component_id_by_name(&self, component_name: &str) -> anyhow::Result<Option<Uuid>> {
        Ok(self
            .service_component_by_name(component_name)
            .await?
            .map(|c| c.versioned_component_id.component_id))
    }
}

impl ComponentCommandHandler for CommandHandler {
    fn base(&self) -> &CommandHandler {
        self
    }

    fn base_mut(&mut self) -> &mut CommandHandler {
        self
    }
}
