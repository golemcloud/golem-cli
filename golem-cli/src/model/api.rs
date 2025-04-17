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

use crate::model::to_oss::ToOss;
use chrono::{DateTime, Utc};
use golem_client::model::{ApiDefinitionInfo, ApiSite, Provider};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum IdentityProviderType {
    Google,
    Facebook,
    Gitlab,
    Microsoft,
}

impl Display for IdentityProviderType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Google => "google",
            Self::Facebook => "facebook",
            Self::Gitlab => "gitlab",
            Self::Microsoft => "microsoft",
        };
        Display::fmt(&s, f)
    }
}

impl FromStr for IdentityProviderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "google" => Ok(IdentityProviderType::Google),
            "facebook" => Ok(IdentityProviderType::Facebook),
            "gitlab" => Ok(IdentityProviderType::Gitlab),
            "microsoft" => Ok(IdentityProviderType::Microsoft),
            _ => Err(format!(
                "Unknown identity provider type: {s}. Expected one of \"google\", \"facebook\", \"gitlab\", \"microsoft\""
            )),
        }
    }
}

impl From<IdentityProviderType> for Provider {
    fn from(value: IdentityProviderType) -> Self {
        match value {
            IdentityProviderType::Google => Provider::Google,
            IdentityProviderType::Facebook => Provider::Facebook,
            IdentityProviderType::Gitlab => Provider::Gitlab,
            IdentityProviderType::Microsoft => Provider::Microsoft,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ApiDefinitionIdWithVersion {
    pub id: ApiDefinitionId,
    pub version: ApiDefinitionVersion,
}

impl Display for ApiDefinitionIdWithVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.id, self.version)
    }
}

impl FromStr for ApiDefinitionIdWithVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('@').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid api definition id with version: {s}. Expected format: <id>@<version>"
            ));
        }

        let id = ApiDefinitionId(parts[0].to_string());
        let version = ApiDefinitionVersion(parts[1].to_string());

        Ok(ApiDefinitionIdWithVersion { id, version })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ApiDefinitionId(pub String);

impl Display for ApiDefinitionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ApiDefinitionId {
    fn from(id: &str) -> Self {
        ApiDefinitionId(id.to_string())
    }
}

impl From<String> for ApiDefinitionId {
    fn from(id: String) -> Self {
        ApiDefinitionId(id)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ApiDefinitionVersion(pub String);

impl Display for ApiDefinitionVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ApiDefinitionVersion {
    fn from(id: &str) -> Self {
        ApiDefinitionVersion(id.to_string())
    }
}

impl From<String> for ApiDefinitionVersion {
    fn from(id: String) -> Self {
        ApiDefinitionVersion(id)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiDeployment {
    #[serde(rename = "apiDefinitions")]
    pub api_definitions: Vec<ApiDefinitionInfo>,
    #[serde(rename = "projectId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub project_id: Option<Uuid>,
    pub site: ApiSite,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
}

impl From<golem_client::model::ApiDeployment> for ApiDeployment {
    fn from(value: golem_client::model::ApiDeployment) -> Self {
        ApiDeployment {
            api_definitions: value.api_definitions,
            project_id: None,
            site: value.site,
            created_at: value.created_at,
        }
    }
}

impl From<golem_cloud_client::model::ApiDeployment> for ApiDeployment {
    fn from(value: golem_cloud_client::model::ApiDeployment) -> Self {
        ApiDeployment {
            api_definitions: value.api_definitions.to_oss(),
            project_id: Some(value.project_id),
            site: value.site.to_oss(),
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiSecurityScheme {
    #[serde(rename = "schemeIdentifier")]
    pub scheme_identifier: String,
    #[serde(rename = "clientId")]
    pub client_id: String,
    #[serde(rename = "clientSecret")]
    pub client_secret: String,
    #[serde(rename = "redirectUrl")]
    pub redirect_url: String,
    pub scopes: Vec<String>,
}

impl From<golem_client::model::SecuritySchemeData> for ApiSecurityScheme {
    fn from(value: golem_client::model::SecuritySchemeData) -> Self {
        ApiSecurityScheme {
            scheme_identifier: value.scheme_identifier,
            client_id: value.client_id,
            client_secret: value.client_secret,
            redirect_url: value.redirect_url,
            scopes: value.scopes,
        }
    }
}

impl From<golem_cloud_client::model::SecuritySchemeData> for ApiSecurityScheme {
    fn from(value: golem_cloud_client::model::SecuritySchemeData) -> Self {
        ApiSecurityScheme {
            scheme_identifier: value.scheme_identifier,
            client_id: value.client_id,
            client_secret: value.client_secret,
            redirect_url: value.redirect_url,
            scopes: value.scopes,
        }
    }
}
