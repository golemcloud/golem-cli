use async_trait::async_trait;
use golem_cloud_client::model::Account;
use golem_cloud_client::model::AccountData;
use golem_cloud_client::model::Plan;
use tracing::info;

use crate::model::{AccountId, GolemError};

#[async_trait]
pub trait AccountClient {
    async fn get(&self, id: &AccountId) -> Result<Account, GolemError>;
    async fn get_plan(&self, id: &AccountId) -> Result<Plan, GolemError>;
    async fn put(&self, id: &AccountId, data: AccountData) -> Result<Account, GolemError>;
    async fn post(&self, data: AccountData) -> Result<Account, GolemError>;
    async fn delete(&self, id: &AccountId) -> Result<(), GolemError>;
}

pub struct AccountClientLive<C: golem_cloud_client::api::AccountClient + Sync + Send> {
    pub client: C,
}

#[async_trait]
impl<C: golem_cloud_client::api::AccountClient + Sync + Send> AccountClient
    for AccountClientLive<C>
{
    async fn get(&self, id: &AccountId) -> Result<Account, GolemError> {
        info!("Getting account {id}");
        Ok(self.client.account_id_get(&id.id).await?)
    }

    async fn get_plan(&self, id: &AccountId) -> Result<Plan, GolemError> {
        info!("Getting account plan of {id}.");
        Ok(self.client.account_id_plan_get(&id.id).await?)
    }

    async fn put(&self, id: &AccountId, data: AccountData) -> Result<Account, GolemError> {
        info!("Updating account {id}.");
        Ok(self.client.account_id_put(&id.id, &data).await?)
    }

    async fn post(&self, data: AccountData) -> Result<Account, GolemError> {
        info!("Creating account.");
        Ok(self.client.post(&data).await?)
    }

    async fn delete(&self, id: &AccountId) -> Result<(), GolemError> {
        info!("Deleting account {id}.");
        let _ = self.client.account_id_delete(&id.id).await?;
        Ok(())
    }
}
