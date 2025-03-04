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

use crate::model::GolemError;
use async_trait::async_trait;

#[async_trait]
pub trait PluginClient {
    type ProjectContext;
    type PluginDefinition;
    type PluginDefinitionWithoutOwner;
    type PluginScope;

    async fn list_plugins(
        &self,
        scope: Option<Self::PluginScope>,
    ) -> Result<Vec<Self::PluginDefinition>, GolemError>;

    async fn get_plugin(
        &self,
        plugin_name: &str,
        plugin_version: &str,
    ) -> Result<Self::PluginDefinition, GolemError>;

    async fn register_plugin(
        &self,
        definition: Self::PluginDefinitionWithoutOwner,
    ) -> Result<Self::PluginDefinition, GolemError>;

    async fn unregister_plugin(
        &self,
        plugin_name: &str,
        plugin_version: &str,
    ) -> Result<(), GolemError>;
}
