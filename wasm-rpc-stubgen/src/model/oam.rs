use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const API_VERSION_V1BETA1: &str = "core.oam.dev/v1beta1";
pub const KIND_APPLICATION: &str = "Application";

#[derive(Clone, Debug)]
pub struct ApplicationWithSource {
    pub source: PathBuf,
    pub application: Application,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Application {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: Metadata,
    pub spec: Spec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub annotations: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub labels: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Spec {
    pub components: Vec<Component>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    #[serde(rename = "type")]
    pub component_type: String,
    pub properties: serde_json::Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<Trait>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trait {
    #[serde(rename = "type")]
    pub trait_type: String,
    pub properties: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::assert;

    #[test]
    fn deserialize_example_application() {
        let application: Application = serde_yaml::from_str(
            r#"
apiVersion: core.oam.dev/v1beta1
metadata:
  name: "App name"
kind: Application
spec:
  components:
    - name: component-one
      type: test-component-type
      properties:
        testComponentProperty: aaa
      traits:
        - type: test-trait-type-1
          properties:
            testProperty: bbb
        - type: test-trait-type-2
          properties:
            testTraitProperty: ccc
"#,
        )
        .unwrap();

        assert!(application.api_version == API_VERSION_V1BETA1);
        assert!(application.kind == KIND_APPLICATION);
        assert!(application.metadata.name == "App name");
        assert!(application.spec.components.len() == 1);

        let component = &application.spec.components[0];

        assert!(component.name == "component-one");
        assert!(component.component_type == "test-component-type");
        assert!(component.properties.is_object());

        let properties = component.properties.as_object().unwrap();

        assert!(
            properties
                .get_key_value("testComponentProperty")
                .unwrap()
                .1
                .as_str()
                == Some("aaa")
        );

        assert!(component.traits.len() == 2);

        let component_trait = &component.traits[1];

        assert!(component_trait.trait_type == "test-trait-type-2");
        assert!(component_trait.properties.is_object());

        let properties = component_trait.properties.as_object().unwrap();

        assert!(properties.get_key_value("testTraitProperty").unwrap().1 == "ccc");
    }
}
