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

use crate::cloud::AccountId;
use crate::command_handler::GetHandler;
use crate::config::ProfileKind;
use crate::context::Context;
use crate::error::service::AnyhowMapServiceError;
use crate::error::{HintError, NonSuccessfulExit};
use crate::model::project::ProjectView;
use crate::model::text::fmt::{log_error, log_text_view};
use crate::model::text::help::ComponentNameHelp;
use crate::model::{ComponentName, ProjectName, ProjectNameAndId};
use anyhow::{anyhow, bail};
use golem_cloud_client::api::ProjectClient;
use golem_wasm_rpc_stubgen::log::{logln, LogColorize};
use std::sync::Arc;

pub struct CloudProjectCommandHandler {
    ctx: Arc<Context>,
}

impl CloudProjectCommandHandler {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    async fn opt_project_by_name(
        &self,
        account_id: Option<&AccountId>,
        project_name: &ProjectName,
    ) -> anyhow::Result<Option<ProjectView>> {
        let mut projects = self
            .ctx
            .golem_clients_cloud()
            .await?
            .project
            .get_projects(Some(&project_name.0))
            .await
            .map_service_error()?
            .into_iter()
            .map(ProjectView::from)
            .collect::<Vec<_>>();

        match account_id {
            Some(account_id) => {
                let project_idx = projects
                    .iter()
                    .position(|project| &project.owner_account_id == account_id);
                match project_idx {
                    Some(project_idx) => Ok(Some(projects.swap_remove(project_idx))),
                    None => Ok(None),
                }
            }
            None => {
                if projects.len() == 1 {
                    Ok(Some(projects.pop().unwrap()))
                } else {
                    log_error(format!(
                        "Project name {} is ambiguous!",
                        project_name.0.log_color_highlight()
                    ));
                    bail!(NonSuccessfulExit)
                }
            }
        }
    }

    pub async fn project_by_name(
        &self,
        account_id: Option<&AccountId>,
        project_name: &ProjectName,
    ) -> anyhow::Result<ProjectView> {
        match self.opt_project_by_name(account_id, &project_name).await? {
            Some(project) => Ok(project),
            None => Err(project_not_found(account_id, project_name)),
        }
    }

    async fn default_project(&self) -> anyhow::Result<ProjectView> {
        Ok(self
            .ctx
            .golem_clients_cloud()
            .await?
            .project
            .get_default_project()
            .await
            .map(ProjectView::from)
            .map_service_error()?)
    }

    // TODO: special care might be needed for ordering app loading if
    //       project selection can be defined if app manifest too
    pub async fn opt_select_project(
        &self,
        account_id: Option<&AccountId>,
        project_name: Option<&ProjectName>,
    ) -> anyhow::Result<Option<ProjectNameAndId>> {
        match (self.ctx.profile_kind(), project_name) {
            (ProfileKind::Oss, Some(_)) => {
                log_error("Cannot use projects with OSS profile!");
                logln("");
                log_text_view(&ComponentNameHelp);
                logln("");
                bail!(NonSuccessfulExit)
            }
            (ProfileKind::Oss, None) => {
                // TODO: from global flags
                Ok(None)
            }
            (ProfileKind::Cloud, Some(project_name)) => {
                let project = self.project_by_name(account_id, project_name).await?;
                Ok(Some(ProjectNameAndId {
                    project_name: project.name,
                    project_id: project.project_id,
                }))
            }
            (ProfileKind::Cloud, None) => {
                // TODO: from global flags
                // TODO: should we query the default here?
                Ok(None)
            }
        }
    }

    pub async fn select_project(
        &self,
        account_id: Option<&AccountId>,
        project_name: &ProjectName,
    ) -> anyhow::Result<ProjectNameAndId> {
        match self
            .opt_select_project(account_id, Some(project_name))
            .await?
        {
            Some(project) => Ok(project),
            None => Err(project_not_found(account_id, project_name)),
        }
    }
}

fn project_not_found(account_id: Option<&AccountId>, project_name: &ProjectName) -> anyhow::Error {
    let formatted_account = account_id
        .map(|id| format!("{} / ", id.0.log_color_highlight()))
        .unwrap_or_default();
    log_error(format!(
        "Project {}{} not found.",
        formatted_account,
        project_name.0.log_color_highlight()
    ));
    anyhow!(NonSuccessfulExit)
}
