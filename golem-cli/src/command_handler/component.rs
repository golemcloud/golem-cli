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
use crate::command_handler::GetHandler;
use crate::context::{Context, GolemClients};
use crate::error::{to_service_error, NonSuccessfulExit};
use crate::model::app_ext::GolemComponentExtensions;
use crate::model::component::{Component, ComponentView};
use crate::model::text::component::{ComponentCreateView, ComponentGetView, ComponentUpdateView};
use crate::model::text::fmt::{log_error, log_warn};
use crate::model::{ComponentName, ComponentNameMatchKind};
use anyhow::{anyhow, bail, Context as AnyhowContext};
use golem_client::api::ComponentClient as ComponentClientOss;
use golem_client::model::DynamicLinkedInstance as DynamicLinkedInstanceOss;
use golem_client::model::DynamicLinkedWasmRpc as DynamicLinkedWasmRpcOss;
use golem_client::model::DynamicLinking as DynamicLinkingOss;
use golem_common::model::ComponentType;
use golem_wasm_rpc_stubgen::commands::app::{ApplicationContext, ComponentSelectMode};
use golem_wasm_rpc_stubgen::log::{log_action, logln, LogColorize, LogIndent};
use golem_wasm_rpc_stubgen::model::app::DependencyType;
use golem_wasm_rpc_stubgen::model::app::{BuildProfileName, ComponentName as AppComponentName};
use itertools::Itertools;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tracing::debug;
use uuid::Uuid;

pub struct ComponentCommandHandler {
    ctx: Arc<Context>,
}

impl ComponentCommandHandler {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    pub async fn handle_command(&mut self, subcommand: ComponentSubcommand) -> anyhow::Result<()> {
        match subcommand {
            ComponentSubcommand::New { .. } => {
                todo!()
            }
            ComponentSubcommand::Build {
                component_name,
                build: build_args,
            } => {
                self.ctx
                    .app_handler()
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
                self.deploy(
                    component_name.component_name,
                    Some(force_build),
                    &ComponentSelectMode::CurrentDir,
                )
                .await
            }
            ComponentSubcommand::Clean { component_name } => {
                self.ctx
                    .app_handler()
                    .clean(
                        component_name.component_name,
                        &ComponentSelectMode::CurrentDir,
                    )
                    .await
            }
            ComponentSubcommand::List { component_name } => {
                self.list(component_name.component_name).await
            }
            ComponentSubcommand::Get {
                component_name,
                version,
            } => self.get(component_name.component_name, version).await,
        }
    }

    pub async fn deploy(
        &mut self,
        component_names: Vec<ComponentName>,
        force_build: Option<ForceBuildArg>,
        default_component_select_mode: &ComponentSelectMode,
    ) -> anyhow::Result<()> {
        self.ctx
            .app_handler()
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

        let selected_component_names = {
            let app_ctx = self.ctx.app_context_lock().await;
            app_ctx
                .some_or_err()?
                .selected_component_names()
                .iter()
                .cloned()
                .collect::<Vec<_>>()
        };
        let build_profile = self.ctx.build_profile().cloned();

        log_action("Deploying", "components");

        for component_name in &selected_component_names {
            let _indent = LogIndent::new();

            let component_id = self.component_id_by_name(component_name.as_str()).await?;
            let deploy_properties = {
                let mut app_ctx = self.ctx.app_context_lock_mut().await;
                let app_ctx = app_ctx.some_or_err_mut()?;
                component_deploy_properties(app_ctx, component_name, build_profile.clone())?
            };

            let linked_wasm = File::open(&deploy_properties.linked_wasm_path)
                .await
                .with_context(|| {
                    anyhow!(
                        "Failed to open component linked WASM at {}",
                        deploy_properties
                            .linked_wasm_path
                            .display()
                            .to_string()
                            .log_color_error_highlight()
                    )
                })?;

            match &component_id {
                Some(component_id) => {
                    log_action(
                        "Updating",
                        format!(
                            "component {}",
                            component_name.as_str().log_color_highlight()
                        ),
                    );
                    let _indent = LogIndent::new();
                    match self.ctx.golem_clients().await? {
                        GolemClients::Oss(clients) => {
                            let component = clients
                                .component
                                .update_component(
                                    component_id,
                                    Some(&deploy_properties.component_type),
                                    linked_wasm,
                                    None,         // TODO:
                                    None::<File>, // TODO:
                                    deploy_properties.dynamic_linking.as_ref(),
                                )
                                .await
                                .map_err(to_service_error)?;
                            self.ctx
                                .log_handler()
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
                    let _indent = self.ctx.log_handler().nested_text_view_indent();
                    match self.ctx.golem_clients().await? {
                        GolemClients::Oss(clients) => {
                            let component = clients
                                .component
                                .create_component(
                                    component_name.as_str(),
                                    Some(&deploy_properties.component_type),
                                    linked_wasm,
                                    None,         // TODO:
                                    None::<File>, // TODO:
                                    deploy_properties.dynamic_linking.as_ref(),
                                )
                                .await
                                .map_err(to_service_error)?;
                            self.ctx
                                .log_handler()
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

    async fn list(&self, component_name: Option<ComponentName>) -> anyhow::Result<()> {
        let selected_component_names = self
            .opt_select_by_app_or_name(component_name.as_ref())
            .await?;

        let mut component_views = Vec::<ComponentView>::new();

        if selected_component_names.is_empty() {
            // TODO: there is no pagination for components
            match self.ctx.golem_clients().await? {
                GolemClients::Oss(clients) => {
                    let results = clients
                        .component
                        .get_components(None)
                        .await
                        .map_err(to_service_error)?;
                    component_views.extend(
                        results
                            .into_iter()
                            .map(|meta| ComponentView::from(Component::from(meta))),
                    );
                }
                GolemClients::Cloud(_) => {
                    todo!()
                }
            }
        } else {
            for component_name in selected_component_names {
                match self.ctx.golem_clients().await? {
                    GolemClients::Oss(clients) => {
                        let results = clients
                            .component
                            .get_components(Some(component_name.as_str()))
                            .await
                            .map_err(to_service_error)?;
                        if results.is_empty() {
                            log_warn(format!(
                                "No versions found for component {}",
                                component_name.as_str().log_color_highlight()
                            ));
                        } else {
                            component_views.extend(
                                results
                                    .into_iter()
                                    .map(|meta| ComponentView::from(Component::from(meta))),
                            );
                        }
                    }
                    GolemClients::Cloud(_) => {
                        todo!()
                    }
                }
            }
        }

        if component_views.is_empty() && component_name.is_some() {
            // Retry selection (this time with not allowing "not founds")
            // so we get error messages for app component names.
            self.ctx
                .app_handler()
                .opt_select_components(
                    component_name.iter().cloned().collect(),
                    &ComponentSelectMode::CurrentDir,
                )
                .await?;
        }

        if component_views.is_empty() {
            bail!(NonSuccessfulExit)
        } else {
            self.ctx.log_handler().log_view(&component_views);
        }

        Ok(())
    }

    async fn get(
        &self,
        component_name: Option<ComponentName>,
        version: Option<u64>,
    ) -> anyhow::Result<()> {
        let selected_component_names = self
            .must_select_by_app_or_name(component_name.as_ref())
            .await?;

        if version.is_some() && selected_component_names.len() > 1 {
            log_error("Version cannot be specific when multiple components are selected!");
            logln("");
            logln(format!(
                "Selected components: {}",
                selected_component_names
                    .iter()
                    .map(|cn| cn.as_str().log_color_highlight())
                    .join(", ")
            ));
            logln("");
            logln("Specify the requested component name or switch to an application directory with exactly one component!");
            logln("");
            bail!(NonSuccessfulExit);
        }

        let mut component_views = Vec::<ComponentView>::new();

        for component_name in selected_component_names {
            match self.component_id_by_name(component_name.as_str()).await? {
                Some(component_id) => match self.ctx.golem_clients().await? {
                    GolemClients::Oss(clients) => match version {
                        Some(version) => {
                            let result = clients
                                .component
                                .get_component_metadata(&component_id, &version.to_string())
                                .await
                                .map_err(to_service_error)?;
                            component_views.push(Component::from(result).into());
                        }
                        None => {
                            let result = clients
                                .component
                                .get_latest_component_metadata(&component_id)
                                .await
                                .map_err(to_service_error)?;
                            component_views.push(Component::from(result).into());
                        }
                    },
                    GolemClients::Cloud(_) => {
                        todo!()
                    }
                },
                None => {
                    log_warn(format!(
                        "Component {} not found",
                        component_name.as_str().log_color_highlight()
                    ));
                }
            }
        }

        // TODO: code dup
        if component_views.is_empty() && component_name.is_some() {
            // Retry selection (this time with not allowing "not founds")
            // so we get error messages for app component names.
            self.ctx
                .app_handler()
                .opt_select_components(
                    component_name.iter().cloned().collect(),
                    &ComponentSelectMode::CurrentDir,
                )
                .await?;
        }

        let no_matches = component_views.is_empty();
        for component_view in component_views {
            self.ctx
                .log_handler()
                .log_view(&ComponentGetView(component_view));
            logln("");
        }

        // TODO: if it was a version request we can try to enumerate valid version numbers
        if no_matches {
            bail!(NonSuccessfulExit)
        }

        Ok(())
    }

    pub async fn opt_select_by_app_or_name(
        &self,
        component_name: Option<&ComponentName>,
    ) -> anyhow::Result<Vec<String>> {
        self.select_by_app_or_name_internal(component_name, true)
            .await
    }

    pub async fn must_select_by_app_or_name(
        &self,
        component_name: Option<&ComponentName>,
    ) -> anyhow::Result<Vec<String>> {
        self.select_by_app_or_name_internal(component_name, false)
            .await
    }

    async fn select_by_app_or_name_internal(
        &self,
        component_name: Option<&ComponentName>,
        allow_no_matches: bool,
    ) -> anyhow::Result<Vec<String>> {
        self.ctx.silence_app_context_init().await;
        let app_select_success = self
            .ctx
            .app_handler()
            .opt_select_components_allow_not_found(
                component_name.into_iter().cloned().collect(),
                &ComponentSelectMode::CurrentDir,
            )
            .await?;

        let selected_component_names = {
            if app_select_success {
                let app_ctx = self.ctx.app_context_lock().await;
                app_ctx
                    .opt()?
                    .map(|app_ctx| {
                        app_ctx
                            .selected_component_names()
                            .iter()
                            .map(|cn| cn.to_string())
                            .collect::<Vec<_>>()
                    })
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
            } else {
                component_name
                    .iter()
                    .map(|cn| cn.0.to_string())
                    .collect::<Vec<_>>()
            }
        };

        if selected_component_names.is_empty() && component_name.is_none() && !allow_no_matches {
            log_error("No components were selected based on the current directory an no component was requested.");
            logln("");
            logln(
                "Please specify a requested component name or switch to an application directory!",
            );
            logln("");
            bail!(NonSuccessfulExit);
        }

        Ok(selected_component_names)
    }

    pub async fn component_by_name_with_auto_deploy(
        &self,
        component_match_kind: ComponentNameMatchKind,
        component_name: &ComponentName,
    ) -> anyhow::Result<Component> {
        match self.component_by_name(&component_name.0).await? {
            Some(component) => Ok(component),
            None => {
                let should_deploy = match component_match_kind {
                    ComponentNameMatchKind::AppCurrentDir => true,
                    ComponentNameMatchKind::App => true,
                    ComponentNameMatchKind::Unknown => false,
                };

                if !should_deploy {
                    logln("");
                    log_error(format!(
                        "Component {} not found, and not part of the current application",
                        component_name.0.log_color_highlight()
                    ));
                    // TODO: fuzzy match from service to list components
                    bail!(NonSuccessfulExit)
                }

                // TODO: we will need hashes to reliably detect if "update" deploy is needed
                //       and for now we should not blindly keep updating, so for now
                //       only missing one are handled
                log_action(
                    "Auto deploying",
                    format!(
                        "missing component {}",
                        component_name.0.log_color_highlight()
                    ),
                );
                self.ctx
                    .component_handler()
                    .deploy(
                        vec![component_name.clone()],
                        None,
                        &ComponentSelectMode::CurrentDir,
                    )
                    .await?;
                self.ctx
                    .component_handler()
                    .component_by_name(&component_name.0)
                    .await?
                    .ok_or_else(|| {
                        anyhow!("Component ({}) not found after deployment", component_name)
                    })
            }
        }
    }

    // TODO: we might want to have a filter for batch name lookups on the server side
    // TODO: also the search returns all versions
    // TODO: maybe add transient or persistent cache for all the meta
    pub async fn component_by_name(
        &self,
        component_name: &str,
    ) -> anyhow::Result<Option<Component>> {
        match self.ctx.golem_clients().await? {
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
            .component_by_name(component_name)
            .await?
            .map(|c| c.versioned_component_id.component_id))
    }
}

// TODO: cloud
struct ComponentDeployProperties {
    component_type: ComponentType,
    linked_wasm_path: PathBuf,
    dynamic_linking: Option<DynamicLinkingOss>,
}

fn component_deploy_properties(
    app_ctx: &mut ApplicationContext<GolemComponentExtensions>,
    component_name: &AppComponentName,
    build_profile: Option<BuildProfileName>,
) -> anyhow::Result<ComponentDeployProperties> {
    let linked_wasm_path = app_ctx
        .application
        .component_linked_wasm(component_name, build_profile.as_ref());
    let component_properties = &app_ctx
        .application
        .component_properties(component_name, build_profile.as_ref());
    let extensions = &component_properties.extensions;
    let component_type = extensions.component_type;
    let dynamic_linking = app_component_dynamic_linking_oss(app_ctx, component_name)?;

    Ok(ComponentDeployProperties {
        component_type,
        linked_wasm_path,
        dynamic_linking,
    })
}

fn app_component_dynamic_linking_oss(
    app_ctx: &mut ApplicationContext<GolemComponentExtensions>,
    component_name: &AppComponentName,
) -> anyhow::Result<Option<DynamicLinkingOss>> {
    let mut mapping = Vec::new();

    let wasm_rpc_deps = app_ctx
        .application
        .component_wasm_rpc_dependencies(component_name)
        .iter()
        .filter(|dep| dep.dep_type == DependencyType::DynamicWasmRpc)
        .cloned()
        .collect::<Vec<_>>();

    for wasm_rpc_dep in wasm_rpc_deps {
        mapping.push(app_ctx.component_stub_interfaces(&wasm_rpc_dep.name)?);
    }

    if mapping.is_empty() {
        Ok(None)
    } else {
        Ok(Some(DynamicLinkingOss {
            dynamic_linking: HashMap::from_iter(mapping.into_iter().map(|stub_interfaces| {
                (
                    stub_interfaces.stub_interface_name,
                    DynamicLinkedInstanceOss::WasmRpc(DynamicLinkedWasmRpcOss {
                        target_interface_name: HashMap::from_iter(
                            stub_interfaces.exported_interfaces_per_stub_resource,
                        ),
                    }),
                )
            })),
        }))
    }
}
