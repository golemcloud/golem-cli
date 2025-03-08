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
use crate::command::shared_args::{BuildArgs, ForceBuildArg};
use crate::command_handler::component::ComponentCommandHandler;
use crate::command_handler::{to_service_error, CommandHandler};
use crate::context::GolemClients;
use crate::model::component::Component;
use crate::model::text::component::{ComponentCreateView, ComponentUpdateView};
use crate::model::ComponentName;
use anyhow::{anyhow, bail, Context};
use golem_client::api::ComponentClient;
use golem_examples::add_component_by_example;
use golem_examples::model::{ComposableAppGroupName, PackageName};
use golem_wasm_rpc_stubgen::commands::app::{ComponentSelectMode, DynamicHelpSections};
use golem_wasm_rpc_stubgen::fs;
use golem_wasm_rpc_stubgen::log::{log_action, logln, LogColorize, LogIndent};
use itertools::Itertools;
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
}

impl AppCommandHandler for CommandHandler {
    fn base(&self) -> &CommandHandler {
        &self
    }

    fn base_mut(&mut self) -> &mut CommandHandler {
        self
    }
}
