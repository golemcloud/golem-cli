use crate::fs;
use crate::log::LogColorize;
use crate::model::component::AppComponentType;
use anyhow::{anyhow, Context};
use golem_common::model::{ComponentFilePath, ComponentFilePermissions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct ApplicationWithSource {
    pub source: PathBuf,
    pub application: Application,
}

impl ApplicationWithSource {
    pub fn from_yaml_file(file: PathBuf) -> anyhow::Result<Self> {
        Self::from_yaml_string(file.clone(), fs::read_to_string(file.clone())?)
            .with_context(|| anyhow!("Failed to load source {}", file.log_color_highlight()))
    }

    pub fn from_yaml_string(source: PathBuf, string: String) -> serde_yaml::Result<Self> {
        Ok(Self {
            source,
            application: Application::from_yaml_str(string.as_str())?,
        })
    }

    pub fn source_as_string(&self) -> String {
        self.source.to_string_lossy().to_string()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Application {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub includes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temp_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wit_deps: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub templates: HashMap<String, ComponentTemplate>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub components: HashMap<String, Component>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub dependencies: HashMap<String, Vec<Dependency>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_commands: HashMap<String, Vec<ExternalCommand>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clean: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub api_definitions: HashMap<String, HttpApiDefinition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub api_deployments: Vec<HttpApiDeployment>,
}

impl Application {
    pub fn from_yaml_str(yaml: &str) -> serde_yaml::Result<Self> {
        serde_yaml::from_str(yaml)
    }

    pub fn to_yaml_string(&self) -> String {
        serde_yaml::to_string(self).expect("Failed to serialize Application as YAML")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ComponentTemplate {
    #[serde(flatten)]
    pub component_properties: ComponentProperties,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub profiles: HashMap<String, ComponentProperties>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_profile: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Component {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(flatten)]
    pub component_properties: ComponentProperties,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub profiles: HashMap<String, ComponentProperties>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_profile: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HttpApi {
    definitions: HashMap<String, HttpApiDefinition>,
    deployments: Vec<HttpApiDeployment>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HttpApiDefinition {
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    pub draft: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routes: Vec<HttpApiDefinitionRoute>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HttpApiDefinitionRoute {
    pub method: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cors: Option<HttpApiCors>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security: Option<String>,
    pub binding: HttpApiDefinitionBinding,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HttpApiCors {
    enabled: bool,
    allow_origin: Option<String>,
    allow_methods: Option<String>,
    allow_headers: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expose_headers: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    allow_credentials: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max_age: Option<u64>,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum HttpApiDefinitionBindingType {
    #[default]
    Default,
    FileServer,
    HttpHandler,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HttpApiDefinitionBinding {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    type_: Option<HttpApiDefinitionBindingType>,
    component_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    component_version: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    worker_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    idempotency_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    response: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    invocation_context: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HttpApiDeployment {
    pub host: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subdomain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub definitions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InitialComponentFile {
    pub source_path: String,
    pub target_path: ComponentFilePath,
    pub permissions: Option<ComponentFilePermissions>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ComponentProperties {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_wit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_wit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_wasm: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linked_wasm: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub build: Vec<ExternalCommand>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_commands: HashMap<String, Vec<ExternalCommand>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clean: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_type: Option<AppComponentType>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<InitialComponentFile>,
}

impl ComponentProperties {
    pub fn defined_property_names(&self) -> Vec<&str> {
        let mut vec = Vec::<&str>::new();

        if self.source_wit.is_some() {
            vec.push("sourceWit");
        }

        if self.generated_wit.is_some() {
            vec.push("generatedWit");
        }

        if self.component_wasm.is_some() {
            vec.push("componentWasm");
        }

        if self.linked_wasm.is_some() {
            vec.push("linkedWasm");
        }

        if !self.build.is_empty() {
            vec.push("build");
        }

        if !self.custom_commands.is_empty() {
            vec.push("customCommands");
        }

        if self.component_type.is_some() {
            vec.push("componentType");
        }

        if !self.files.is_empty() {
            vec.push("files");
        }

        vec
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExternalCommand {
    pub command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rmdirs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mkdirs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Dependency {
    #[serde(rename = "type")]
    pub type_: String,
    pub target: Option<String>,
}
