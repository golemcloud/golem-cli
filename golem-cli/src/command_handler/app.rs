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

use crate::command::app::AppSubcommand;
use crate::command::shared_args::BuildArgs;
use crate::command_handler::component::ComponentCommandHandler;
use crate::command_handler::CommandHandler;
use crate::error::NonSuccessfulExit;
use crate::fuzzy::{Error, FuzzySearch};
use crate::model::app_ext::GolemComponentExtensions;
use crate::model::text::fmt::{log_error, log_text_view};
use crate::model::text::help::AvailableComponentNamesHelp;
use crate::model::ComponentName;
use anyhow::{anyhow, bail};
use colored::Colorize;
use golem_client::model::DynamicLinkedInstance as DynamicLinkedInstanceOss;
use golem_client::model::DynamicLinkedWasmRpc as DynamicLinkedWasmRpcOss;
use golem_client::model::DynamicLinking as DynamicLinkingOss;
use golem_examples::add_component_by_example;
use golem_examples::model::{ComposableAppGroupName, PackageName};
use golem_wasm_rpc_stubgen::commands::app::{
    ApplicationContext, ComponentSelectMode, DynamicHelpSections,
};
use golem_wasm_rpc_stubgen::fs;
use golem_wasm_rpc_stubgen::log::{log_action, logln, LogColorize, LogIndent, LogOutput, Output};
use golem_wasm_rpc_stubgen::model::app::ComponentName as AppComponentName;
use golem_wasm_rpc_stubgen::model::app::DependencyType;
use itertools::Itertools;
use std::collections::HashMap;
use std::path::PathBuf;

pub trait AppCommandHandler {
    fn base(&self) -> &CommandHandler;
    fn base_mut(&mut self) -> &mut CommandHandler;

    async fn handle_app_subcommand(&mut self, subcommand: AppSubcommand) -> anyhow::Result<()> {
        match subcommand {
            AppSubcommand::New {
                application_name,
                language,
            } => {
                let app_dir = PathBuf::from(&application_name);
                if app_dir.exists() {
                    bail!(
                        "Application directory already exists: {}",
                        app_dir.log_color_error_highlight()
                    );
                }

                // TODO: check for no parent manifests

                fs::create_dir_all(&app_dir)?;
                log_action(
                    "Created",
                    format!(
                        "application directory: {}",
                        app_dir.display().to_string().log_color_highlight()
                    ),
                );

                {
                    let _indent = LogIndent::new();
                    for language in language.language {
                        let Some(language_examples) = self.base().ctx.templates().get(&language)
                        else {
                            bail!(
                                "No template found for {}, currently supported languages: {}",
                                language.to_string().log_color_error_highlight(),
                                self.base().ctx.templates().keys().join(", ")
                            );
                        };

                        let default_examples = language_examples
                            .get(&ComposableAppGroupName::default())
                            .expect("No default template found for the selected language");

                        // TODO:
                        assert_eq!(
                            default_examples.components.len(),
                            1,
                            "Expected exactly one default component template"
                        );
                        let (_, default_component_example) =
                            &default_examples.components.iter().next().unwrap();

                        // TODO: better default names
                        let component_package_name = PackageName::from_string(format!(
                            "sample:{}",
                            language.to_string().to_lowercase()
                        ))
                        .unwrap(); // TODO: from args optionally

                        match add_component_by_example(
                            default_examples.common.as_ref(),
                            default_component_example,
                            &app_dir,
                            &component_package_name,
                        ) {
                            Ok(()) => {
                                log_action(
                                    "Added",
                                    format!(
                                        "new app component: {}",
                                        component_package_name
                                            .to_string_with_colon()
                                            .log_color_highlight()
                                    ),
                                );
                            }
                            Err(error) => {
                                bail!("Failed to add new app component: {}", error)
                            }
                        }
                    }
                }

                std::env::set_current_dir(&app_dir)?;
                let Some(app_ctx) = self.base().ctx.application_context().await? else {
                    return Ok(());
                };

                logln("");
                app_ctx.log_dynamic_help(&DynamicHelpSections {
                    components: true,
                    custom_commands: true,
                })?;

                Ok(())
            }
            AppSubcommand::Build {
                component_name,
                build: build_args,
            } => {
                self.build(
                    component_name.component_name,
                    Some(build_args),
                    &ComponentSelectMode::All,
                )
                .await
            }
            AppSubcommand::Deploy {
                component_name,
                force_build,
            } => {
                self.base_mut()
                    .deploy(
                        component_name.component_name,
                        Some(force_build),
                        &ComponentSelectMode::All,
                    )
                    .await
            }
            AppSubcommand::Clean { component_name } => {
                self.clean(component_name.component_name, &ComponentSelectMode::All)
                    .await
            }
            AppSubcommand::CustomCommand(command) => {
                if command.len() != 1 {
                    bail!(
                        "Expected exactly one custom subcommand, got: {}",
                        command.join(" ").log_color_error_highlight()
                    );
                }

                self.base()
                    .required_application_context()
                    .await?
                    .custom_command(&command[0])?;

                Ok(())
            }
        }
    }

    async fn build(
        &mut self,
        component_names: Vec<ComponentName>,
        build: Option<BuildArgs>,
        default_component_select_mode: &ComponentSelectMode,
    ) -> anyhow::Result<()> {
        if let Some(build) = build {
            self.base_mut()
                .ctx
                .set_steps_filter(build.step.into_iter().collect());
            self.base_mut()
                .ctx
                .set_skip_up_to_date_checks(build.force_build.force_build);
        }

        self.base_mut()
            .required_app_ctx_with_selection_mut(component_names, default_component_select_mode)
            .await?
            .build()
            .await?;

        Ok(())
    }

    async fn clean(
        &mut self,
        component_names: Vec<ComponentName>,
        default_component_select_mode: &ComponentSelectMode,
    ) -> anyhow::Result<()> {
        self.base_mut()
            .required_app_ctx_with_selection_mut(component_names, default_component_select_mode)
            .await?
            .clean()?;

        Ok(())
    }

    async fn required_application_context(
        &self,
    ) -> anyhow::Result<&ApplicationContext<GolemComponentExtensions>> {
        self.base()
            .ctx
            .application_context()
            .await?
            .ok_or_else(no_application_manifest_found_error)
    }

    async fn required_application_context_mut(
        &mut self,
    ) -> anyhow::Result<&mut ApplicationContext<GolemComponentExtensions>> {
        self.base_mut()
            .ctx
            .application_context_mut()
            .await?
            .ok_or_else(no_application_manifest_found_error)
    }

    async fn required_app_ctx_with_selection_mut(
        &mut self,
        component_names: Vec<ComponentName>,
        default: &ComponentSelectMode,
    ) -> anyhow::Result<&mut ApplicationContext<GolemComponentExtensions>> {
        self.app_ctx_with_selection_mut(component_names, default)
            .await?
            .ok_or_else(no_application_manifest_found_error)
    }

    // TODO: forbid matching the same component multiple times
    async fn app_ctx_with_selection_mut(
        &mut self,
        component_names: Vec<ComponentName>,
        default: &ComponentSelectMode,
    ) -> anyhow::Result<Option<&mut ApplicationContext<GolemComponentExtensions>>> {
        let silent_selection = self.base_mut().ctx.silent_application_context_init();
        let Some(app_ctx) = self.base_mut().ctx.application_context_mut().await? else {
            return Ok(None);
        };

        if component_names.is_empty() {
            let _log_output = silent_selection.then(|| LogOutput::new(Output::None));
            app_ctx.select_components(default)?
        } else {
            let fuzzy_search =
                FuzzySearch::new(app_ctx.application.component_names().map(|cn| cn.as_str()));

            let (found, not_found) =
                fuzzy_search.find_many(component_names.iter().map(|cn| cn.0.as_str()));

            if !not_found.is_empty() {
                logln("");
                log_error(format!(
                    "The following requested component names were not found:\n{}",
                    not_found
                        .iter()
                        .map(|error| {
                            match error {
                                Error::Ambiguous {
                                    pattern,
                                    highlighted_options,
                                } => {
                                    format!(
                                        "  - {}, did you mean one of {}?",
                                        pattern.as_str().bold(),
                                        highlighted_options.iter().map(|cn| cn.bold()).join(", ")
                                    )
                                }
                                Error::NotFound { pattern } => {
                                    format!("  - {}", pattern.as_str().bold())
                                }
                            }
                        })
                        .join("\n")
                ));
                logln("");
                log_text_view(&AvailableComponentNamesHelp(
                    app_ctx.application.component_names().cloned().collect(),
                ));

                bail!(NonSuccessfulExit);
            }

            let _log_output = silent_selection.then(|| LogOutput::new(Output::None));
            app_ctx.select_components(&ComponentSelectMode::Explicit(
                found.into_iter().map(|m| m.option.into()).collect(),
            ))?
        }
        Ok(Some(app_ctx))
    }

    async fn app_component_dynamic_linking_oss(
        &mut self,
        component_name: &AppComponentName,
    ) -> anyhow::Result<Option<DynamicLinkingOss>> {
        let app_ctx = self.required_application_context_mut().await?;

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
}

impl AppCommandHandler for CommandHandler {
    fn base(&self) -> &CommandHandler {
        &self
    }

    fn base_mut(&mut self) -> &mut CommandHandler {
        self
    }
}

fn no_application_manifest_found_error() -> anyhow::Error {
    logln("");
    log_error("No application manifest(s) found!");
    logln(format!(
        "Switch to a directory that contains an application manifest ({}),",
        "golem.yaml".log_color_highlight()
    ));
    logln(format!(
        "or create a new application with the '{}' subcommand!",
        "app new".log_color_highlight(),
    ));
    anyhow!(NonSuccessfulExit)
}
