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
use crate::command_handler::app::AppCommandHandler;
use crate::command_handler::component::ComponentCommandHandler;
use crate::command_handler::worker::WorkerCommandHandler;
use crate::command_handler::{CommandHandler, GetHandler};
use crate::context::Context;
use crate::model::component::show_exported_functions;
use crate::model::text::fmt::{log_text_view, NestedTextViewIndent};
use crate::model::text::help::{AvailableFunctionNamesHelp, WorkerNameHelp};
use crate::model::ComponentNameMatchKind;
use colored::Colorize;
use golem_wasm_rpc_stubgen::commands::app::{ComponentSelectMode, DynamicHelpSections};
use golem_wasm_rpc_stubgen::log::{log_action, logln, LogColorize};
use std::sync::Arc;

pub struct PartialMatchHandler {
    ctx: Arc<Context>,
}

impl PartialMatchHandler {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    pub(crate) async fn handle_partial_match(
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
                for (language, templates) in self.ctx.templates() {
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
                self.ctx.silence_app_context_init();
                self.ctx
                    .app_handler()
                    .opt_select_app_components(vec![], &ComponentSelectMode::All)?;

                let app_ctx = self.ctx.app_context();
                if let Some(app_ctx) = app_ctx.opt()? {
                    logln("");
                    app_ctx.log_dynamic_help(&DynamicHelpSections {
                        components: true,
                        custom_commands: true,
                    })?
                } else {
                    // TODO: maybe add hint that this command should use app manifest
                }

                Ok(())
            }
            GolemCliCommandPartialMatch::ComponentMissingSubcommandHelp => {
                self.ctx.silence_app_context_init();
                self.ctx
                    .app_handler()
                    .opt_select_app_components(vec![], &ComponentSelectMode::All)?;

                let app_ctx = self.ctx.app_context();
                if let Some(app_ctx) = app_ctx.opt()? {
                    logln("");
                    app_ctx.log_dynamic_help(&DynamicHelpSections {
                        components: true,
                        custom_commands: false,
                    })?
                } else {
                    // TODO: maybe add hint that this command should use app manifest
                }

                Ok(())
            }
            GolemCliCommandPartialMatch::WorkerInvokeMissingWorkerName => {
                logln("");
                log_text_view(&WorkerNameHelp);
                logln("");
                // TODO: maybe also show available component names from app?
                Ok(())
            }
            GolemCliCommandPartialMatch::WorkerInvokeMissingFunctionName { worker_name } => {
                self.ctx.silence_app_context_init();
                logln("");
                log_action(
                    "Checking",
                    format!(
                        "provided worker name: {}",
                        worker_name.0.log_color_highlight()
                    ),
                );
                let component_name = {
                    let _indent = NestedTextViewIndent::new();
                    let (component_name_match_kind, component_name, worker_name) = self
                        .ctx
                        .worker_handler()
                        .match_worker_name(worker_name)
                        .await?;
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
                    .ctx
                    .component_handler()
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
