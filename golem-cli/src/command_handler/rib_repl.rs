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

use crate::command_handler::Handlers;
use crate::context::Context;
use crate::log::LogColorize;
use crate::model::app::ApplicationComponentSelectMode;
use crate::model::{ComponentName, ComponentNameMatchKind, IdempotencyKey, WorkerName};
use anyhow::format_err;
use async_trait::async_trait;
use golem_rib_repl::dependency_manager::{
    ReplDependencies, RibComponentMetadata, RibDependencyManager,
};
use golem_rib_repl::invoke::WorkerFunctionInvoke;
use golem_rib_repl::rib_repl::RibRepl;
use golem_wasm_rpc::json::OptionallyTypeAnnotatedValueJson;
use golem_wasm_rpc::ValueAndType;
use rib::{EvaluatedFnArgs, EvaluatedFqFn, EvaluatedWorkerName};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

pub struct RibReplHandler {
    ctx: Arc<Context>,
}

impl RibReplHandler {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    pub async fn run_repl(
        &self,
        component_names: Vec<ComponentName>,
        component_select_mode: &ApplicationComponentSelectMode,
    ) -> anyhow::Result<()> {
        self.ctx
            .app_handler()
            .must_select_components(component_names, component_select_mode)
            .await?;

        let mut repl = RibRepl::bootstrap(
            None, // TODO
            Arc::new(self.ctx.rib_repl_handler()),
            Arc::new(self.ctx.rib_repl_handler()),
            None, // TODO?
            None,
        )
        .await
        .map_err(|err| format_err!("{:?}", err))?; // TODO: use display once implemented

        repl.run().await;

        Ok(())
    }
}

#[async_trait]
impl RibDependencyManager for RibReplHandler {
    async fn get_dependencies(&self) -> Result<ReplDependencies, String> {
        let component_name = {
            let app_ctx = self.ctx.app_context_lock().await;
            let app_ctx = app_ctx.some_or_err().map_err(|err| err.to_string())?; // TODO: anyhow

            let mut selected_component_names = app_ctx
                .selected_component_names()
                .iter()
                .map(|cn| cn.as_str().into())
                .collect::<Vec<ComponentName>>();

            if selected_component_names.len() != 1 {
                /* TODO: once we have anyhow
                bail!("Only one component is supported")
                */
                return Err("Only one component is supported".to_string());
            }

            selected_component_names.pop().unwrap()
        };

        let component = self
            .ctx
            .component_handler()
            .component_by_name_with_auto_deploy(
                None, // TODO: project
                ComponentNameMatchKind::App,
                &component_name,
                None,
            )
            .await
            .map_err(|err| err.to_string())?; // TODO: anyhow

        Ok(ReplDependencies {
            component_dependencies: vec![RibComponentMetadata {
                // TODO: name
                component_id: component.versioned_component_id.component_id,
                metadata: component.metadata.exports,
            }],
        })
    }

    async fn add_component(
        &self,
        source_path: &Path,
        component_name: String,
    ) -> Result<RibComponentMetadata, String> {
        unreachable!("add_component is not available in CLI")
    }
}

#[async_trait]
impl WorkerFunctionInvoke for RibReplHandler {
    async fn invoke(
        &self,
        component_id: Uuid, // TODO: let's add component name too for debug purposes
        worker_name: Option<EvaluatedWorkerName>,
        function_name: EvaluatedFqFn,
        args: EvaluatedFnArgs,
    ) -> Result<ValueAndType, String> {
        let worker_name: Option<WorkerName> = worker_name.as_ref().map(|wn| wn.0.as_str().into());

        let component = self
            .ctx
            .component_handler()
            .component(
                None, // TODO
                component_id.into(),
                worker_name.as_ref(),
            )
            .await
            .map_err(|err| err.to_string())?; // TODO: fix once repl is using anyhow

        let Some(component) = component else {
            return Err("Component not found".to_string());
            /* TODO:
            log_error("Component not found"); // TODO: show component name once we have it
            bail!(NonSuccessfulExit);
            */
        };

        let arguments: Vec<OptionallyTypeAnnotatedValueJson> = args
            .0
            .into_iter()
            .map(|vat| vat.try_into().unwrap())
            .collect();

        let result = self
            .ctx
            .worker_handler()
            .invoke_worker(
                &component,
                worker_name.as_ref(),
                &function_name.0,
                arguments,
                IdempotencyKey::new(),
                false,
                None,
            )
            .await
            .map_err(|err| err.to_string())?
            .unwrap(); // TODO: fix once repl is using anyhow;

        result.result.try_into()
    }
}
