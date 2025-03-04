use crate::cloud::clients::errors::CloudGolemError;
use async_trait::async_trait;
use golem_cli::cloud::{AccountId, ProjectId};
use golem_cloud_client::model::{Project, ProjectDataRequest};
use golem_common::uri::cloud::urn::ProjectUrn;
use tracing::info;

#[async_trait]
pub trait ProjectClient {
    async fn create(
        &self,
        owner_account_id: &AccountId,
        name: String,
        description: Option<String>,
    ) -> Result<Project, CloudGolemError>;
    async fn find(&self, name: Option<String>) -> Result<Vec<Project>, CloudGolemError>;
    async fn find_default(&self) -> Result<Project, CloudGolemError>;
    async fn get(&self, urn: ProjectUrn) -> Result<Project, CloudGolemError>;
    async fn delete(&self, project_id: ProjectId) -> Result<(), CloudGolemError>;
}

pub struct ProjectClientLive<C: golem_cloud_client::api::ProjectClient + Sync + Send> {
    pub client: C,
}

#[async_trait]
impl<C: golem_cloud_client::api::ProjectClient + Sync + Send> ProjectClient
    for ProjectClientLive<C>
{
    async fn create(
        &self,
        owner_account_id: &AccountId,
        name: String,
        description: Option<String>,
    ) -> Result<Project, CloudGolemError> {
        info!("Create new project {name}.");

        let request = ProjectDataRequest {
            name,
            owner_account_id: owner_account_id.id.to_string(),
            description: description.unwrap_or("".to_string()),
        };
        Ok(self.client.create_project(&request).await?)
    }

    async fn find(&self, name: Option<String>) -> Result<Vec<Project>, CloudGolemError> {
        info!("Listing projects.");

        Ok(self.client.get_projects(name.as_deref()).await?)
    }

    async fn find_default(&self) -> Result<Project, CloudGolemError> {
        info!("Getting default project.");

        Ok(self.client.get_default_project().await?)
    }

    async fn get(&self, urn: ProjectUrn) -> Result<Project, CloudGolemError> {
        info!("Getting project {urn}.");

        Ok(self.client.get_project(&urn.id.0).await?)
    }

    async fn delete(&self, project_id: ProjectId) -> Result<(), CloudGolemError> {
        info!("Deleting project {project_id:?}");

        let _ = self.client.delete_project(&project_id.0).await?;

        Ok(())
    }
}
