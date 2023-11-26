use async_trait::async_trait;

use crate::clients::project::ProjectClient;
use crate::clients::project_grant::ProjectGrantClient;
use crate::model::{
    AccountId, GolemError, GolemResult, ProjectAction, ProjectId, ProjectPolicyId, ProjectRef,
};

#[async_trait]
pub trait ProjectGrantHandler {
    async fn handle(
        &self,
        project_ref: ProjectRef,
        recipient_account_id: AccountId,
        project_policy_id: Option<ProjectPolicyId>,
        project_actions: Option<Vec<ProjectAction>>,
    ) -> Result<GolemResult, GolemError>;
}
pub struct ProjectGrantHandlerLive<
    'p,
    C: ProjectGrantClient + Send + Sync,
    P: ProjectClient + Sync + Send,
> {
    pub client: C,
    pub project: &'p P,
}

#[async_trait]
impl<'p, C: ProjectGrantClient + Send + Sync, P: ProjectClient + Sync + Send> ProjectGrantHandler
    for ProjectGrantHandlerLive<'p, C, P>
{
    async fn handle(
        &self,
        project_ref: ProjectRef,
        recipient_account_id: AccountId,
        project_policy_id: Option<ProjectPolicyId>,
        project_actions: Option<Vec<ProjectAction>>,
    ) -> Result<GolemResult, GolemError> {
        let project_id = match self.project.resolve_id(project_ref).await? {
            None => ProjectId(self.project.find_default().await?.project_id),
            Some(id) => id,
        };
        match project_policy_id {
            None => {
                let actions = project_actions.unwrap();

                let grant = self
                    .client
                    .create_actions(project_id, recipient_account_id, actions)
                    .await?;

                Ok(GolemResult::Ok(Box::new(grant)))
            }
            Some(policy_id) => {
                let grant = self
                    .client
                    .create(project_id, recipient_account_id, policy_id)
                    .await?;

                Ok(GolemResult::Ok(Box::new(grant)))
            }
        }
    }
}
