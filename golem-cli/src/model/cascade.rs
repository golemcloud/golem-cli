// Copyright 2024-2025 Golem Cloud
//
// Licensed under the Golem Source License v1.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://license.golem.cloud/LICENSE
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub trait Selector: Clone + Eq + Hash {}

pub struct Store<L: Layer> {
    layers: HashMap<L::Id, L>,
    value_cache: RefCell<HashMap<L::Id, HashMap<L::Selector, L::Value>>>,
}

#[derive(Debug, thiserror::Error)]
pub enum StoreGetValueError<L: Layer> {
    #[error("requested layer not found: {0}")]
    LayerNotFound(L::Id),
    #[error("layer ({0}) apply error: {1}")]
    LayerApplyError(L::Id, L::ApplyError),
}

#[derive(Debug, thiserror::Error)]
pub enum StoreAddLayerError<L: Layer> {
    #[error("layer already exists: {0}")]
    LayerAlreadyExists(L::Id),
}

// TODO: check for circular parents (either on add or on get, or have a separate validation step)
impl<L: Layer> Store<L> {
    pub fn new() -> Store<L> {
        Self {
            layers: HashMap::new(),
            value_cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn add_layer(&mut self, layer: L) -> Result<(), StoreAddLayerError<L>> {
        if self.layers.contains_key(&layer.id()) {
            return Err(StoreAddLayerError::LayerAlreadyExists(layer.id().clone()));
        }
        self.layers.insert(layer.id().clone(), layer);
        Ok(())
    }

    pub fn value(
        &self,
        id: &L::Id,
        selector: &L::Selector,
    ) -> Result<Ref<L::Value>, StoreGetValueError<L>> {
        {
            let value_cache = self.value_cache.borrow();
            if value_cache
                .get(id)
                .map(|v| v.contains_key(selector))
                .unwrap_or(false)
            {
                return Ok(Ref::map(value_cache, |value_cache| {
                    value_cache.get(id).unwrap().get(selector).unwrap()
                }));
            }
        }

        let Some(layer) = self.layers.get(id) else {
            return Err(StoreGetValueError::LayerNotFound(id.clone()));
        };

        fn apply_layer<L: Layer>(
            store: &Store<L>,
            selector: &L::Selector,
            layer: &L,
            value: &mut L::Value,
        ) -> Result<(), StoreGetValueError<L>> {
            for layer_id in layer.parent_layers() {
                let Some(layer) = store.layers.get(layer_id) else {
                    return Err(StoreGetValueError::LayerNotFound(layer_id.clone()));
                };
                apply_layer(store, selector, layer, value)?;
            }
            if let Some(err) = layer.apply_onto_parent(selector, value).err() {
                return Err(StoreGetValueError::LayerApplyError(layer.id().clone(), err));
            };
            Ok(())
        }
        let mut value = L::Value::default();
        apply_layer(self, selector, layer, &mut value)?;

        {
            let mut value_cache = self.value_cache.borrow_mut();
            let value_cache = match value_cache.get_mut(id) {
                Some(value_cache) => value_cache,
                None => {
                    value_cache.insert(id.clone(), HashMap::new());
                    value_cache.get_mut(id).unwrap()
                }
            };
            value_cache.insert(selector.clone(), value);
        }

        Ok(Ref::map(self.value_cache.borrow(), |value_cache| {
            value_cache.get(id).unwrap().get(selector).unwrap()
        }))
    }
}

pub trait Layer {
    type Id: Debug + Eq + Hash + Clone + Serialize;
    type Value: Debug + Default + Clone + Serialize;
    type Selector: Debug + Eq + Hash + Clone;
    type AppliedSelection: Debug + Clone + Serialize;
    type ApplyError;

    fn id(&self) -> &Self::Id;
    fn parent_layers(&self) -> &[Self::Id];
    fn apply_onto_parent(
        &self,
        selector: &Self::Selector,
        value: &mut Self::Value,
    ) -> Result<(), Self::ApplyError>;
}

pub mod property {
    use crate::model::cascade::Layer;

    pub trait Property<L: Layer> {
        type Value;
        type PropertyLayer;
        type TraceElem;

        fn value(&self) -> &Self::Value;
        fn trace(&self) -> &[Self::TraceElem];

        fn apply_layer(
            &mut self,
            id: &L::Id,
            selection: Option<&L::AppliedSelection>,
            layer: Self::PropertyLayer,
        );
    }

    pub mod optional {
        use crate::model::cascade::property::Property;
        use crate::model::cascade::Layer;
        use serde_derive::Serialize;

        #[derive(Debug, Clone, Serialize)]
        #[serde(tag = "type", rename_all = "kebab-case")]
        pub enum OptionalPropertyTraceElem<L: Layer, V> {
            #[serde(rename_all = "camelCase")]
            Override {
                id: L::Id,
                selection: Option<L::AppliedSelection>,
                value: V,
            },
            #[serde(rename_all = "camelCase")]
            Skip {
                id: L::Id,
                selection: Option<L::AppliedSelection>,
            },
        }

        #[derive(Debug, Clone, Serialize)]
        #[serde(rename_all = "camelCase")]
        pub struct OptionalProperty<L: Layer, V> {
            value: Option<V>,
            trace: Vec<OptionalPropertyTraceElem<L, V>>,
        }

        impl<L: Layer, V> Default for OptionalProperty<L, V> {
            fn default() -> Self {
                Self::none()
            }
        }

        impl<L: Layer, V> OptionalProperty<L, V> {
            pub fn new(value: Option<V>) -> Self {
                Self {
                    value,
                    trace: vec![],
                }
            }

            pub fn some(value: V) -> Self {
                Self::new(Some(value))
            }

            pub fn none() -> Self {
                Self::new(None)
            }
        }

        impl<L: Layer, V> From<V> for OptionalProperty<L, V> {
            fn from(value: V) -> Self {
                Self::new(Some(value))
            }
        }

        impl<V: Clone, L: Layer> From<&V> for OptionalProperty<L, V> {
            fn from(value: &V) -> Self {
                Self::new(Some(value.to_owned()))
            }
        }

        impl<L: Layer, V> From<Option<V>> for OptionalProperty<L, V> {
            fn from(value: Option<V>) -> Self {
                Self::new(value)
            }
        }

        impl<V: Clone, L: Layer> From<Option<&V>> for OptionalProperty<L, V> {
            fn from(value: Option<&V>) -> Self {
                value.map(|v| v.clone()).into()
            }
        }

        impl<V: Clone, L: Layer> Property<L> for OptionalProperty<L, V> {
            type Value = Option<V>;
            type PropertyLayer = Option<V>;
            type TraceElem = OptionalPropertyTraceElem<L, V>;

            fn value(&self) -> &Self::Value {
                &self.value
            }

            fn trace(&self) -> &[Self::TraceElem] {
                self.trace.as_slice()
            }

            fn apply_layer(
                &mut self,
                id: &L::Id,
                selection: Option<&L::AppliedSelection>,
                layer: Self::PropertyLayer,
            ) {
                match layer {
                    Some(value) => {
                        self.value = Some(value.clone());
                        self.trace.push(OptionalPropertyTraceElem::Override {
                            id: id.clone(),
                            selection: selection.map(|selection| selection.clone()),
                            value,
                        })
                    }
                    None => self.trace.push(OptionalPropertyTraceElem::Skip {
                        id: id.clone(),
                        selection: selection.map(|selection| selection.clone()),
                    }),
                }
            }
        }
    }

    pub mod map {
        use crate::model::cascade::property::Property;
        use crate::model::cascade::{Layer, Selector};
        use serde::Serialize;
        use std::collections::HashMap;
        use std::hash::Hash;

        #[derive(Debug, Clone, Copy, Default, Serialize)]
        #[serde(rename_all = "kebab-case")]
        pub enum MapMergeMode {
            #[default]
            Upsert,
            Replace,
            Remove,
        }

        #[derive(Debug, Clone, Serialize)]
        #[serde(tag = "type", rename_all = "kebab-case")]
        pub enum MapPropertyTraceElem<L: Layer, K, V> {
            #[serde(rename_all = "camelCase")]
            Upsert {
                id: L::Id,
                selection: Option<L::AppliedSelection>,
                inserted_entries: HashMap<K, V>,
                updated_entries: HashMap<K, V>,
            },
            #[serde(rename_all = "camelCase")]
            Replace {
                id: L::Id,
                selection: Option<L::AppliedSelection>,
                new_entries: HashMap<K, V>,
            },
            #[serde(rename_all = "camelCase")]
            Remove {
                id: L::Id,
                selection: Option<L::AppliedSelection>,
                removed_entries: HashMap<K, V>,
            },
        }

        #[derive(Debug, Clone, Serialize)]
        #[serde(rename_all = "camelCase")]
        pub struct MapProperty<L: Layer, K: Serialize, V: Serialize> {
            map: HashMap<K, V>,
            trace: Vec<MapPropertyTraceElem<L, K, V>>,
        }

        impl<L: Layer, K: Serialize, V: Serialize> Default for MapProperty<L, K, V> {
            fn default() -> Self {
                Self {
                    map: HashMap::new(),
                    trace: vec![],
                }
            }
        }

        impl<L: Layer, K: Serialize, V: Serialize> MapProperty<L, K, V> {
            pub fn new(map: HashMap<K, V>) -> Self {
                Self { map, trace: vec![] }
            }

            // TODO: from and from_iter, like HashMap
        }

        impl<L: Layer, K: Serialize, V: Serialize> From<HashMap<K, V>> for MapProperty<L, K, V> {
            fn from(value: HashMap<K, V>) -> Self {
                Self::new(value)
            }
        }

        impl<L: Layer, K: Eq + Hash + Clone + Serialize, V: Clone + Serialize> Property<L>
            for MapProperty<L, K, V>
        {
            type Value = HashMap<K, V>;
            type PropertyLayer = (MapMergeMode, HashMap<K, V>);
            type TraceElem = MapPropertyTraceElem<L, K, V>;

            fn value(&self) -> &Self::Value {
                &self.map
            }

            fn trace(&self) -> &[Self::TraceElem] {
                self.trace.as_slice()
            }

            fn apply_layer(
                &mut self,
                id: &L::Id,
                selection: Option<&L::AppliedSelection>,
                layer: Self::PropertyLayer,
            ) {
                let (mode, map) = layer;
                match mode {
                    MapMergeMode::Upsert => {
                        let mut inserted_entries = HashMap::new();
                        let mut updated_entries = HashMap::new();
                        for (key, value) in map {
                            if self.map.insert(key.clone(), value.clone()).is_some() {
                                updated_entries.insert(key, value);
                            } else {
                                inserted_entries.insert(key, value);
                            }
                        }
                        self.trace.push(MapPropertyTraceElem::Upsert {
                            id: id.clone(),
                            selection: selection.map(|selection| selection.clone()),
                            inserted_entries,
                            updated_entries,
                        });
                    }
                    MapMergeMode::Replace => {
                        self.map = map.clone();
                        self.trace.push(MapPropertyTraceElem::Replace {
                            id: id.clone(),
                            selection: selection.map(|selection| selection.clone()),
                            new_entries: map,
                        })
                    }
                    MapMergeMode::Remove => {
                        let mut removed_entries = HashMap::new();
                        for (key, _) in map {
                            if let Some(value) = self.map.remove(&key) {
                                removed_entries.insert(key, value);
                            }
                        }
                        self.trace.push(MapPropertyTraceElem::Remove {
                            id: id.clone(),
                            selection: selection.map(|selection| selection.clone()),
                            removed_entries,
                        })
                    }
                }
            }
        }
    }

    pub mod vec {
        #[derive(Debug, Clone, Copy, Default)]
        pub enum VecMergeMode {
            #[default]
            Append,
            Prepend,
            Replace,
        }
    }
}

#[cfg(test)]
mod test {
    mod example_component_properties {
        use crate::model::cascade::property::map::{MapMergeMode, MapProperty};
        use crate::model::cascade::property::optional::OptionalProperty;
        use crate::model::cascade::property::Property;
        use crate::model::cascade::test::example_component_properties::ComponentLayerId::{
            BaseDefinition, BaseTemplate, DefinitionPresets, TemplatePresets,
        };
        use crate::model::cascade::{Layer, Store};
        use crate::model::deploy_diff::ToYamlValueWithoutNulls;
        use serde_derive::Serialize;
        use std::collections::HashMap;
        use test_r::test;

        #[derive(Debug, Eq, Hash, PartialEq, Clone, Serialize)]
        #[serde(rename_all = "kebab-case")]
        enum ComponentLayerId {
            BaseTemplate(String),
            TemplatePresets(String),
            BaseDefinition(String),
            DefinitionPresets(String),
        }

        impl ComponentLayerId {
            pub fn is_template(&self) -> bool {
                match self {
                    BaseTemplate(_) => true,
                    TemplatePresets(_) => true,
                    BaseDefinition(_) => false,
                    DefinitionPresets(_) => false,
                }
            }
        }

        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        struct ComponentSelector {
            pub selected_presets: Vec<String>,
            pub template_env: Vec<(String, String)>,
        }

        #[derive(Debug, Default, Clone, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct ComponentProperties {
            pub component_type: OptionalProperty<ComponentLayer, String>,
            pub build: OptionalProperty<ComponentLayer, String>,
            pub env: MapProperty<ComponentLayer, String, String>,
            pub env_merge: Option<MapMergeMode>,
        }

        #[derive(Debug, Clone, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct ComponentLayer {
            id: ComponentLayerId,
            parents: Vec<ComponentLayerId>,
            base_properties: Option<ComponentProperties>,
            preset_properties: HashMap<String, ComponentProperties>,
            default_preset: Option<String>,
        }

        #[derive(Debug, Clone, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct ComponentAppliedSelection {
            pub preset: Option<String>,
            pub used_template_env: Option<Vec<(String, String)>>,
        }

        impl ComponentAppliedSelection {
            pub fn is_empty(&self) -> bool {
                self.preset.is_none() && self.used_template_env.is_none()
            }
        }

        impl Layer for ComponentLayer {
            type Id = ComponentLayerId;
            type Value = ComponentProperties;
            type Selector = ComponentSelector;
            type AppliedSelection = ComponentAppliedSelection;
            type ApplyError = String;

            fn id(&self) -> &Self::Id {
                &self.id
            }

            fn parent_layers(&self) -> &[Self::Id] {
                self.parents.as_slice()
            }

            fn apply_onto_parent(
                &self,
                selector: &Self::Selector,
                value: &mut Self::Value,
            ) -> Result<(), String> {
                let Some((properties, preset)) = (match &self.id {
                    BaseTemplate(_) | BaseDefinition(_) => self
                        .base_properties
                        .as_ref()
                        .map(|properties| (properties, None)),
                    TemplatePresets(_) | DefinitionPresets(_) => selector
                        .selected_presets
                        .iter()
                        .find_map(|preset| {
                            self.preset_properties
                                .get(preset)
                                .map(|properties| (properties, Some(preset)))
                        })
                        .or_else(|| {
                            self.default_preset.as_ref().and_then(|preset| {
                                self.preset_properties
                                    .get(preset)
                                    .map(|properties| (properties, Some(preset)))
                            })
                        }),
                }) else {
                    return Ok(());
                };

                let id = self.id();

                let used_template_env = {
                    if id.is_template() {
                        Some(&selector.template_env)
                    } else {
                        None
                    }
                };

                let templated_selection = ComponentAppliedSelection {
                    preset: preset.map(|preset| preset.clone()),
                    used_template_env: used_template_env.cloned(),
                };
                let templated_selection =
                    (!templated_selection.is_empty()).then_some(&templated_selection);

                let simple_selection = ComponentAppliedSelection {
                    preset: preset.map(|preset| preset.clone()),
                    used_template_env: None,
                };
                let simple_selection = (!simple_selection.is_empty()).then_some(&simple_selection);

                value.component_type.apply_layer(
                    id,
                    simple_selection,
                    properties.component_type.value().clone(),
                );
                value.build.apply_layer(
                    id,
                    templated_selection,
                    properties
                        .build
                        .value()
                        .clone()
                        .map(|build| match used_template_env {
                            Some(used_template_env) => {
                                format!("{}: {:?}", build, used_template_env)
                            }
                            None => build,
                        }),
                );
                value.env.apply_layer(
                    id,
                    simple_selection,
                    (
                        properties.env_merge.unwrap_or_default(),
                        properties.env.value().clone(),
                    ),
                );

                Ok(())
            }
        }

        #[test]
        fn example() {
            let store = {
                let mut store = Store::<ComponentLayer>::new();

                {
                    store
                        .add_layer(ComponentLayer {
                            id: BaseTemplate("rust".to_string()),
                            parents: vec![],
                            base_properties: Some(ComponentProperties {
                                component_type: OptionalProperty::none(),
                                build: OptionalProperty::none(),
                                env: Default::default(),
                                env_merge: None,
                            }),
                            preset_properties: Default::default(),
                            default_preset: None,
                        })
                        .unwrap();

                    store
                        .add_layer(ComponentLayer {
                            id: TemplatePresets("rust".to_string()),
                            parents: vec![BaseTemplate("rust".to_string())],
                            base_properties: None,
                            preset_properties: HashMap::from([
                                (
                                    "debug".to_string(),
                                    ComponentProperties {
                                        component_type: "durable".to_string().into(),
                                        build: "build-debug".to_string().into(),
                                        env: HashMap::from([("X".to_string(), "x".to_string())])
                                            .into(),
                                        env_merge: None,
                                    },
                                ),
                                (
                                    "release".to_string(),
                                    ComponentProperties {
                                        component_type: "ephemeral".to_string().into(),
                                        build: "build-release".to_string().into(),
                                        env: Default::default(),
                                        env_merge: None,
                                    },
                                ),
                            ]),
                            default_preset: Some("debug".to_string()),
                        })
                        .unwrap();
                }

                {
                    store
                        .add_layer(ComponentLayer {
                            id: BaseTemplate("common-env".to_string()),
                            parents: vec![],
                            base_properties: Some(ComponentProperties {
                                component_type: Default::default(),
                                build: OptionalProperty::none(),
                                env: HashMap::from([(
                                    "COMMON_ENV".to_string(),
                                    "common_env".to_string(),
                                )])
                                .into(),
                                env_merge: None,
                            }),
                            preset_properties: Default::default(),
                            default_preset: None,
                        })
                        .unwrap();
                }

                {
                    store
                        .add_layer(ComponentLayer {
                            id: BaseDefinition("app:comp-a".to_string()),
                            parents: vec![TemplatePresets("rust".to_string())],
                            base_properties: None,
                            preset_properties: Default::default(),
                            default_preset: None,
                        })
                        .unwrap();

                    store
                        .add_layer(ComponentLayer {
                            id: BaseDefinition("app:comp-b".to_string()),
                            parents: vec![
                                TemplatePresets("rust".to_string()),
                                BaseTemplate("common-env".to_string()),
                            ],
                            base_properties: None,
                            preset_properties: Default::default(),
                            default_preset: None,
                        })
                        .unwrap();
                }

                store
            };

            let comp = store
                .value(
                    &BaseDefinition("app:comp-b".to_string()),
                    &ComponentSelector {
                        selected_presets: vec!["release".to_string()],
                        template_env: vec![("componentName".to_string(), "appCompB".to_string())],
                    },
                )
                .unwrap();

            println!(
                "{}",
                serde_yaml::to_string(
                    &serde_yaml::to_value(&comp.clone())
                        .unwrap()
                        .to_yaml_value_without_nulls()
                        .unwrap()
                )
                .unwrap()
            )
        }
    }
}
