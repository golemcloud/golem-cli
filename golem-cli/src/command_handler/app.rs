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
use crate::command_handler::GetHandler;
use crate::context::Context;
use crate::error::{HintError, NonSuccessfulExit};
use crate::fuzzy::{Error, FuzzySearch};
use crate::model::text::fmt::{log_error, log_text_view};
use crate::model::text::help::AvailableComponentNamesHelp;
use crate::model::ComponentName;
use anyhow::{anyhow, bail};
use colored::Colorize;
use golem_examples::add_component_by_example;
use golem_examples::model::{ComposableAppGroupName, PackageName};
use golem_wasm_rpc_stubgen::commands::app::{ComponentSelectMode, DynamicHelpSections};
use golem_wasm_rpc_stubgen::fs;
use golem_wasm_rpc_stubgen::log::{log_action, logln, LogColorize, LogIndent, LogOutput, Output};
use itertools::Itertools;
use std::path::PathBuf;
use std::sync::Arc;

pub struct AppCommandHandler {
    ctx: Arc<Context>,
}

impl AppCommandHandler {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    pub(crate) async fn handle_command(&mut self, subcommand: AppSubcommand) -> anyhow::Result<()> {
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
                        let Some(language_examples) = self.ctx.templates().get(&language) else {
                            bail!(
                                "No template found for {}, currently supported languages: {}",
                                language.to_string().log_color_error_highlight(),
                                self.ctx.templates().keys().join(", ")
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
                let app_ctx = self.ctx.app_context_lock();
                let Some(app_ctx) = app_ctx.opt()? else {
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
                self.ctx
                    .component_handler()
                    .deploy(
                        component_name.component_name,
                        Some(force_build),
                        &ComponentSelectMode::All,
                    )
                    .await
            }
            AppSubcommand::Clean { component_name } => {
                self.clean(component_name.component_name, &ComponentSelectMode::All)
            }
            AppSubcommand::CustomCommand(command) => {
                if command.len() != 1 {
                    bail!(
                        "Expected exactly one custom subcommand, got: {}",
                        command.join(" ").log_color_error_highlight()
                    );
                }

                let app_ctx = self.ctx.app_context_lock();
                app_ctx.some_or_err()?.custom_command(&command[0])?;

                Ok(())
            }
        }
    }

    pub async fn build(
        &mut self,
        component_names: Vec<ComponentName>,
        build: Option<BuildArgs>,
        default_component_select_mode: &ComponentSelectMode,
    ) -> anyhow::Result<()> {
        if let Some(build) = build {
            self.ctx.set_steps_filter(build.step.into_iter().collect());
            self.ctx
                .set_skip_up_to_date_checks(build.force_build.force_build);
        }
        self.must_select_components(component_names, default_component_select_mode)?;
        let mut app_ctx = self.ctx.app_context_lock_mut();
        app_ctx.some_or_err_mut()?.build().await
    }

    pub(crate) fn clean(
        &mut self,
        component_names: Vec<ComponentName>,
        default_component_select_mode: &ComponentSelectMode,
    ) -> anyhow::Result<()> {
        self.must_select_components(component_names, default_component_select_mode)?;
        let app_ctx = self.ctx.app_context_lock();
        app_ctx.some_or_err()?.clean()
    }

    fn must_select_components(
        &mut self,
        component_names: Vec<ComponentName>,
        default: &ComponentSelectMode,
    ) -> anyhow::Result<()> {
        self.opt_select_components(component_names, default)?
            .then_some(())
            .ok_or(anyhow!(HintError::NoApplicationManifestFound))
    }

    pub(crate) fn opt_select_components(
        &mut self,
        component_names: Vec<ComponentName>,
        default: &ComponentSelectMode,
    ) -> anyhow::Result<bool> {
        self.opt_select_components_internal(component_names, default, false)
    }

    pub(crate) fn opt_select_components_allow_not_found(
        &mut self,
        component_names: Vec<ComponentName>,
        default: &ComponentSelectMode,
    ) -> anyhow::Result<bool> {
        self.opt_select_components_internal(component_names, default, true)
    }

    // TODO: forbid matching the same component multiple times
    pub(crate) fn opt_select_components_internal(
        &mut self,
        component_names: Vec<ComponentName>,
        default: &ComponentSelectMode,
        allow_not_found: bool,
    ) -> anyhow::Result<bool> {
        let mut app_ctx = self.ctx.app_context_lock_mut();
        let silent_selection = app_ctx.silent_init;
        let Some(app_ctx) = app_ctx.opt_mut()? else {
            return Ok(false);
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
                if allow_not_found {
                    return Ok(false);
                }

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
        Ok(true)
    }
}
