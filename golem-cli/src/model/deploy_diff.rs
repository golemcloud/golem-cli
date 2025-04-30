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

// NOTE: This module contains normalized entities for doing diffs before deployment.
//       This solution is intended to be a naive and temporary one until environments
//       and atomic deployments will be developed.

use crate::log::LogColorize;
use crate::model::api::to_method_pattern;
use crate::model::app::HttpApiDefinitionName;
use crate::model::app_raw::{
    HttpApiDefinition, HttpApiDefinitionBindingType, HttpApiDefinitionRoute,
};
use crate::model::text::fmt::format_rib_source_for_error;
use crate::model::ComponentName;
use anyhow::anyhow;
use golem_client::model::{
    GatewayBindingComponent, GatewayBindingData, GatewayBindingType, HttpApiDefinitionRequest,
    HttpApiDefinitionResponseData, RouteRequestData,
};
use golem_common::model::{ComponentFilePermissions, ComponentType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeployableComponentFile {
    pub hash: String,
    pub permissions: ComponentFilePermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeployableComponent {
    pub component_name: ComponentName,
    pub component_hash: String,
    pub component_type: ComponentType,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub files: BTreeMap<String, DeployableComponentFile>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dynamic_linking: BTreeMap<String, BTreeMap<String, String>>,
}

// TODO: add DeployableHttpApiDefinition, currently using HttpApiDefinitionRequest
pub trait AsHttpApiDefinitionRequest {
    fn as_http_api_definition_request(&self) -> anyhow::Result<HttpApiDefinitionRequest>;
}

impl AsHttpApiDefinitionRequest for HttpApiDefinitionResponseData {
    fn as_http_api_definition_request(&self) -> anyhow::Result<HttpApiDefinitionRequest> {
        Ok(HttpApiDefinitionRequest {
            id: self.id.clone(),
            version: self.version.clone(),
            security: None, // TODO: check that this is not needed anymore
            routes: self
                .routes
                .iter()
                .map(|route| RouteRequestData {
                    method: route.method.clone(),
                    path: route.path.clone(),
                    binding: GatewayBindingData {
                        binding_type: route.binding.binding_type.clone(),
                        component: route.binding.component.as_ref().map(|component| {
                            GatewayBindingComponent {
                                name: component.name.clone(),
                                version: None, // TODO: None for now, how to handle diff on this?
                            }
                        }),
                        worker_name: route.binding.worker_name.clone(),
                        idempotency_key: route.binding.idempotency_key.clone(),
                        response: route.binding.response.clone(),
                        invocation_context: route.binding.invocation_context.clone(),
                    },
                    security: route.security.clone(),
                })
                .collect(),
            draft: self.draft,
        })
    }
}

// TODO: wrapper for the tuple (especially once CORS representation is finalised)
impl AsHttpApiDefinitionRequest for (&HttpApiDefinitionName, &HttpApiDefinition) {
    fn as_http_api_definition_request(&self) -> anyhow::Result<HttpApiDefinitionRequest> {
        let (name, api_definition) = self;

        Ok(HttpApiDefinitionRequest {
            id: name.to_string(),
            version: api_definition.version.clone(),
            security: None, // TODO: check that this is not needed anymore
            routes: api_definition
                .routes
                .iter()
                .map(normalize_http_api_route)
                .collect::<Result<Vec<_>, _>>()?,
            draft: api_definition.draft,
        })
    }
}

fn normalize_http_api_route(route: &HttpApiDefinitionRoute) -> anyhow::Result<RouteRequestData> {
    Ok(RouteRequestData {
        method: to_method_pattern(&route.method)?,
        path: normalize_http_api_binding_path(&route.path),
        binding: GatewayBindingData {
            binding_type: Some(
                route
                    .binding
                    .type_
                    .as_ref()
                    .map(|binding_type| match binding_type {
                        HttpApiDefinitionBindingType::Default => GatewayBindingType::Default,
                        HttpApiDefinitionBindingType::CorsPreflight => {
                            GatewayBindingType::CorsPreflight
                        }
                        HttpApiDefinitionBindingType::FileServer => GatewayBindingType::FileServer,
                        HttpApiDefinitionBindingType::HttpHandler => {
                            GatewayBindingType::HttpHandler
                        }
                    })
                    .unwrap_or_else(|| GatewayBindingType::Default),
            ),
            component: {
                // TODO: how we should handle versions
                route
                    .binding
                    .component_name
                    .as_ref()
                    .map(|name| GatewayBindingComponent {
                        name: name.clone(),
                        version: route.binding.component_version,
                    })
            },
            worker_name: None,
            idempotency_key: normalize_rib_property(&route.binding.idempotency_key)?,
            invocation_context: normalize_rib_property(&route.binding.invocation_context)?,
            response: normalize_rib_property(&route.binding.response)?,
        },
        security: route.security.clone(),
    })
}

fn normalize_rib_property(rib: &Option<String>) -> anyhow::Result<Option<String>> {
    rib.as_ref()
        .map(|r| r.as_str())
        .map(normalize_rib_source_code)
        .transpose()
}

pub fn normalize_http_api_binding_path(path: &str) -> String {
    path.to_string()
}

fn normalize_rib_source_code(rib: &str) -> anyhow::Result<String> {
    Ok(rib::from_string(rib)
        .map_err(|err| {
            anyhow!(
                "Failed to normalize Rib source code: {}\n{}\n{}",
                err,
                "Rib source:".log_color_highlight(),
                format_rib_source_for_error(&err, rib)
            )
        })?
        .to_string())
}

pub trait ToYamlValueWithoutNulls {
    fn to_yaml_value_without_nulls(self) -> serde_yaml::Result<serde_yaml::Value>;
}

impl<T: Serialize> ToYamlValueWithoutNulls for T {
    fn to_yaml_value_without_nulls(self) -> serde_yaml::Result<serde_yaml::Value> {
        Ok(yaml_value_without_nulls(serde_yaml::to_value(self)?))
    }
}

fn yaml_value_without_nulls(value: serde_yaml::Value) -> serde_yaml::Value {
    match value {
        serde_yaml::Value::Mapping(mapping) => serde_yaml::Value::Mapping(
            mapping
                .into_iter()
                .filter_map(|(key, value)| {
                    if value == serde_yaml::Value::Null {
                        None
                    } else {
                        Some((key, yaml_value_without_nulls(value)))
                    }
                })
                .collect(),
        ),
        serde_yaml::Value::Sequence(sequence) => serde_yaml::Value::Sequence(
            sequence.into_iter().map(yaml_value_without_nulls).collect(),
        ),
        _ => value,
    }
}
