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

use crate::command::GolemCliCommandPartialMatch;
use crate::command_handler::component::ComponentCommandHandler;
use crate::command_handler::worker::WorkerCommandHandler;
use crate::command_handler::{log_text_view, CommandHandler, ComponentNameMatchKind};
use crate::model::component::show_exported_functions;
use crate::model::text::help::{AvailableFunctionNamesHelp, WorkerNameHelp};
use colored::Colorize;
use golem_wasm_rpc_stubgen::commands::app::{ComponentSelectMode, DynamicHelpSections};
use golem_wasm_rpc_stubgen::log::{log_action, logln, LogColorize};

pub trait PartialMatchHandler {
    fn base(&self) -> &CommandHandler;
    fn base_mut(&mut self) -> &mut CommandHandler;

    async fn handle_partial_match(
        &mut self,
        partial_match: GolemCliCommandPartialMatch,
    ) -> anyhow::Result<()> {
        match partial_match {
            GolemCliCommandPartialMatch::AppNewMissingLanguage
            | GolemCliCommandPartialMatch::ComponentNewMissingLanguage => {
                logln(format!(
                    "\n{}",
                    "Available languages and templates:".underline().bold(),
                ));
                for (language, templates) in self.base().ctx.templates() {
                    logln(format!("- {}", language.to_string().bold()));
                    for (group, template) in templates {
                        if group.as_str() != "default" {
                            panic!("TODO: handle non-default groups")
                        }
                        // TODO: strip template names (preferably in golem-examples)
                        for template in template.components.values() {
                            logln(format!(
                                "  - {}: {}",
                                template.name.as_str().bold(),
                                template.description,
                            ));
                        }
                    }
                }
                Ok(())
            }
            GolemCliCommandPartialMatch::AppMissingSubcommandHelp => {
                self.base_mut().ctx.silence_application_context_init();
                logln("");
                match self
                    .base_mut()
                    .app_ctx_with_selection_mut(vec![], &ComponentSelectMode::All)
                    .await?
                {
                    Some(app_ctx) => app_ctx.log_dynamic_help(&DynamicHelpSections {
                        components: true,
                        custom_commands: true,
                    }),
                    None => {
                        // TODO: maybe add hint that this command should use app manifest
                        Ok(())
                    }
                }
            }
            GolemCliCommandPartialMatch::ComponentMissingSubcommandHelp => {
                self.base_mut().ctx.silence_application_context_init();
                logln("");
                match self
                    .base_mut()
                    .app_ctx_with_selection_mut(vec![], &ComponentSelectMode::All)
                    .await?
                {
                    Some(app_ctx) => app_ctx.log_dynamic_help(&DynamicHelpSections {
                        components: true,
                        custom_commands: false,
                    }),
                    None => {
                        // TODO: maybe add hint that this command should use app manifest
                        Ok(())
                    }
                }
            }
            GolemCliCommandPartialMatch::WorkerInvokeMissingWorkerName => {
                logln("");
                log_text_view(&WorkerNameHelp);
                logln("");
                // TODO: maybe also show available component names from app?
                Ok(())
            }
            GolemCliCommandPartialMatch::WorkerInvokeMissingFunctionName { worker_name } => {
                self.base_mut().ctx.silence_application_context_init();
                logln("");
                log_action(
                    "Checking",
                    format!(
                        "provided worker name: {}",
                        worker_name.0.log_color_highlight()
                    ),
                );
                let component_name = {
                    let _indent = CommandHandler::nested_text_view_indent();
                    let (component_name_match_kind, component_name, worker_name) =
                        self.base_mut().match_worker_name(worker_name).await?;
                    logln(format!(
                        "[{}] component name: {} / worker_name: {}, {}",
                        "ok".green(),
                        component_name.0.log_color_highlight(),
                        worker_name
                            .as_ref()
                            .map(|s| s.0.as_str())
                            .unwrap_or("-")
                            .log_color_highlight(),
                        match component_name_match_kind {
                            ComponentNameMatchKind::AppCurrentDir =>
                                "component was selected based on current dir",
                            ComponentNameMatchKind::App =>
                                "component was selected from current application",
                            ComponentNameMatchKind::Unknown => "",
                        }
                    ));
                    component_name
                };
                logln("");
                if let Ok(Some(component)) = self
                    .base()
                    .service_component_by_name(&component_name.0)
                    .await
                {
                    log_text_view(&AvailableFunctionNamesHelp {
                        component_name: component_name.0,
                        function_names: show_exported_functions(&component.metadata.exports),
                    });
                    logln("");
                }
                Ok(())
            }
        }
    }
}

impl PartialMatchHandler for CommandHandler {
    fn base(&self) -> &CommandHandler {
        self
    }

    fn base_mut(&mut self) -> &mut CommandHandler {
        self
    }
}
