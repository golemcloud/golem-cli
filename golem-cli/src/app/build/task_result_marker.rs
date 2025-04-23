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

use crate::fs;
use crate::log::log_warn_action;
use crate::model::app::{AppComponentName, DependentComponent};
use crate::model::app_raw;
use anyhow::{anyhow, Context};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use wit_parser::PackageName;

pub trait TaskResultMarkerHashInput {
    fn kind() -> &'static str;

    fn hash_input(&self) -> anyhow::Result<String>;

    /// If returns None, then all_hash_input will be used as id.
    /// The difference between id and hash_input is that id should not include
    /// "task property". E.g.: the hash_input for component dependencies should contain
    /// the dependency names and type, while the id should not. We use this to prevent
    /// detecting false UP-TO-DATE-ness e.g. one keep changing the dependency type.
    fn id(&self) -> anyhow::Result<Option<String>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskResult {
    // NOTE: kind is optional, only used for debugging
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    // NOTE: id is optional, only used for debugging
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    // NOTE: hash_input is optional, only used for debugging
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash_input: Option<String>,

    pub hash_hex: String,
    pub success: bool,
}

#[derive(Serialize)]
pub struct ResolvedExternalCommandMarkerHash<'a> {
    pub build_dir: &'a Path,
    pub command: &'a app_raw::ExternalCommand,
}

impl TaskResultMarkerHashInput for ResolvedExternalCommandMarkerHash<'_> {
    fn kind() -> &'static str {
        "ResolvedExternalCommandMarkerHash"
    }

    fn hash_input(&self) -> anyhow::Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }

    fn id(&self) -> anyhow::Result<Option<String>> {
        Ok(None)
    }
}

pub struct ComponentGeneratorMarkerHash<'a> {
    pub component_name: &'a AppComponentName,
    pub generator_kind: &'a str,
}

impl TaskResultMarkerHashInput for ComponentGeneratorMarkerHash<'_> {
    fn kind() -> &'static str {
        "ComponentGeneratorMarkerHash"
    }

    fn hash_input(&self) -> anyhow::Result<String> {
        Ok(format!("{}-{}", self.component_name, self.generator_kind))
    }

    fn id(&self) -> anyhow::Result<Option<String>> {
        Ok(None)
    }
}

pub struct LinkRpcMarkerHash<'a> {
    pub component_name: &'a AppComponentName,
    pub dependencies: &'a BTreeSet<&'a DependentComponent>,
}

impl TaskResultMarkerHashInput for LinkRpcMarkerHash<'_> {
    fn kind() -> &'static str {
        "RpcLinkMarkerHash"
    }

    fn hash_input(&self) -> anyhow::Result<String> {
        Ok(format!(
            "{}#{}",
            self.component_name,
            self.dependencies
                .iter()
                .map(|s| format!("{}#{}", s.name.as_str(), s.dep_type.as_str()))
                .join(",")
        ))
    }

    fn id(&self) -> anyhow::Result<Option<String>> {
        Ok(Some(self.component_name.to_string()))
    }
}

pub struct AddMetadataMarkerHash<'a> {
    pub component_name: &'a AppComponentName,
    pub root_package_name: PackageName,
}

impl TaskResultMarkerHashInput for AddMetadataMarkerHash<'_> {
    fn kind() -> &'static str {
        "AddMetadataMarkerHash"
    }

    fn hash_input(&self) -> anyhow::Result<String> {
        Ok(format!(
            "{}#{}",
            self.component_name, self.root_package_name
        ))
    }

    fn id(&self) -> anyhow::Result<Option<String>> {
        Ok(Some(self.component_name.to_string()))
    }
}

pub struct TaskResultMarker {
    kind: &'static str,
    id: String,
    hash_input: String,
    marker_file_path: PathBuf,
    hex_hash: String,
    previous_result: Option<TaskResult>,
}

impl TaskResultMarker {
    pub fn new<T: TaskResultMarkerHashInput>(dir: &Path, task: T) -> anyhow::Result<Self> {
        let hash_input = task.hash_input()?;
        let hex_hash = {
            let mut hasher = blake3::Hasher::new();
            hasher.update(T::kind().as_bytes());
            hasher.update(hash_input.as_bytes());
            hasher.finalize().to_hex().to_string()
        };

        let (id, id_hex_hash) = {
            match task.id()? {
                Some(id) => {
                    let mut hasher = blake3::Hasher::new();
                    hasher.update(T::kind().as_bytes());
                    hasher.update(id.as_bytes());
                    (id, hasher.finalize().to_hex().to_string())
                }
                None => (hash_input.clone(), hex_hash.clone()),
            }
        };

        let marker_file_path = dir.join(&id_hex_hash);
        let marker_file_exists = marker_file_path.exists();
        let previous_result = {
            if marker_file_exists {
                match serde_json::from_str::<TaskResult>(&fs::read_to_string(&marker_file_path)?) {
                    Ok(result) => Some(result),
                    Err(err) => {
                        log_warn_action(
                            "Ignoring",
                            format!(
                                "invalid task marker {}: {}",
                                marker_file_path.display(),
                                err
                            ),
                        );
                        None
                    }
                }
            } else {
                None
            }
        };

        let task_result_marker = Self {
            kind: T::kind(),
            id,
            hash_input,
            marker_file_path,
            hex_hash,
            previous_result,
        };

        if marker_file_exists && !task_result_marker.is_up_to_date() {
            fs::remove(&task_result_marker.marker_file_path)?;
        }

        Ok(task_result_marker)
    }

    pub fn is_up_to_date(&self) -> bool {
        match &self.previous_result {
            Some(previous_result) => {
                previous_result.hash_hex == self.hex_hash && previous_result.success
            }
            None => false,
        }
    }

    pub fn success(self) -> anyhow::Result<()> {
        self.save_marker_file(true)
    }

    pub fn failure(self) -> anyhow::Result<()> {
        self.save_marker_file(false)
    }

    fn save_marker_file(self, success: bool) -> anyhow::Result<()> {
        fs::write_str(
            &self.marker_file_path,
            &serde_json::to_string(&TaskResult {
                // TODO: setting kind, id and hash_input could be driven by a debug flag, env or  build
                kind: Some(self.kind.to_string()),
                id: Some(self.id),
                hash_input: Some(self.hash_input),
                hash_hex: self.hex_hash,
                success,
            })?,
        )
    }

    pub fn result<T>(self, result: anyhow::Result<T>) -> anyhow::Result<T> {
        match result {
            Ok(result) => {
                self.success()?;
                Ok(result)
            }
            Err(source_err) => {
                self.failure().with_context(|| {
                    anyhow!(
                        "Failed to save failure marker for source error: {:?}",
                        source_err,
                    )
                })?;
                Err(source_err)
            }
        }
    }
}
