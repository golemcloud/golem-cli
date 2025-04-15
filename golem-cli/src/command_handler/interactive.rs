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

use crate::config::{
    CloudProfile, OssProfile, Profile, ProfileConfig, ProfileKind, ProfileName, CLOUD_URL,
    DEFAULT_OSS_URL,
};
use crate::context::Context;
use crate::error::NonSuccessfulExit;
use crate::log::{log_action, log_warn_action, logln, LogColorize};
use crate::model::app::{AppComponentName, DependencyType};
use crate::model::component::AppComponentType;
use crate::model::text::fmt::{log_error, log_warn};
use crate::model::{ComponentName, Format};
use anyhow::bail;
use colored::Colorize;
use golem_cloud_client::model::Account;
use inquire::error::InquireResult;
use inquire::validator::{ErrorMessage, Validation};
use inquire::{Confirm, CustomType, InquireError, Select, Text};
use itertools::Itertools;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use strum::IntoEnumIterator;
use url::Url;

// NOTE: in the interactive handler is it okay to _read_ context (e.g. read selected component names)
//       but mutations of state or the app should be done in other handlers
pub struct InteractiveHandler {
    ctx: Arc<Context>,
}

impl InteractiveHandler {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    pub fn confirm_auto_deploy_component(
        &self,
        component_name: &ComponentName,
    ) -> anyhow::Result<bool> {
        self.confirm(
            true,
            format!(
                "Component {} was not found between deployed components, do you want to deploy it, then continue?",
                component_name.0.log_color_highlight()
            ),
        )
    }

    pub fn confirm_redeploy_workers(&self, number_of_workers: usize) -> anyhow::Result<bool> {
        self.confirm(
            true,
            format!(
                "Redeploying will {} then recreate {} worker(s), do you want to continue?",
                "delete".log_color_warn(),
                number_of_workers.to_string().log_color_highlight()
            ),
        )
    }

    pub fn confirm_delete_account(&self, account: &Account) -> anyhow::Result<bool> {
        self.confirm(
            false,
            format!(
                "Are you sure you want to delete the requested account? ({}, {})",
                account.name.log_color_highlight(),
                account.email.log_color_highlight()
            ),
        )
    }

    pub fn create_profile(&self) -> anyhow::Result<(ProfileName, Profile, bool)> {
        if !self.confirm(
            true,
            concat!(
                "Do you want to create a new profile interactively?\n",
                "If not, please specify the profile name as a command argument."
            ),
        )? {
            bail!(NonSuccessfulExit);
        }

        let profile_name = Text::new("Profile Name: ")
            .with_validator(|value: &str| {
                if ProfileName::from(value).is_builtin() {
                    return Ok(Validation::Invalid(ErrorMessage::from(
                        "The requested profile name is a builtin one, please choose another name!",
                    )));
                }
                Ok(Validation::Valid)
            })
            .prompt()?;

        let profile_kind =
            Select::new("Profile kind:", ProfileKind::iter().collect::<Vec<_>>()).prompt()?;

        let component_service_url = CustomType::<Url>::new("Component service URL:")
            .with_default(match profile_kind {
                ProfileKind::Oss => Url::parse(DEFAULT_OSS_URL)?,
                ProfileKind::Cloud => Url::parse(CLOUD_URL)?,
            })
            .prompt()?;

        let worker_service_url = CustomType::<OptionalUrl>::new(
            "Worker service URL (empty to use component service url):",
        )
        .prompt()?
        .0;

        let cloud_service_url = match profile_kind {
            ProfileKind::Oss => None,
            ProfileKind::Cloud => {
                CustomType::<OptionalUrl>::new(
                    "Cloud service URL (empty to use component service url)",
                )
                .prompt()?
                .0
            }
        };

        let default_format =
            Select::new("Default output format:", Format::iter().collect::<Vec<_>>())
                .with_starting_cursor(2)
                .prompt()?;

        let profile = match profile_kind {
            ProfileKind::Oss => Profile::Golem(OssProfile {
                url: component_service_url,
                worker_url: worker_service_url,
                allow_insecure: false,
                config: ProfileConfig { default_format },
            }),
            ProfileKind::Cloud => Profile::GolemCloud(CloudProfile {
                custom_url: Some(component_service_url),
                custom_cloud_url: cloud_service_url,
                custom_worker_url: worker_service_url,
                allow_insecure: false,
                config: ProfileConfig { default_format },
                auth: None,
            }),
        };

        let set_as_active = Confirm::new("Set as active profile?")
            .with_default(false)
            .prompt()?;

        Ok((profile_name.into(), profile, set_as_active))
    }

    // TODO: select_component_for_repl, should use app_ctx and filtering
    pub fn select_component(
        &self,
        component_names: Vec<ComponentName>,
    ) -> anyhow::Result<ComponentName> {
        Ok(Select::new(
            "Select a component to be used in Rib REPL:",
            component_names,
        )
        .prompt()?)
    }

    pub async fn create_component_dependency(
        &self,
        component_name: Option<AppComponentName>,
        target_component_name: Option<AppComponentName>,
        dependency_type: Option<DependencyType>,
    ) -> anyhow::Result<Option<(AppComponentName, AppComponentName, DependencyType)>> {
        let component_type_by_name =
            async |component_name: &AppComponentName| -> anyhow::Result<AppComponentType> {
                let app_ctx = self.ctx.app_context_lock().await;
                let app_ctx = app_ctx.some_or_err()?;
                Ok(app_ctx
                    .application
                    .component_properties(&component_name, self.ctx.build_profile())
                    .component_type)
            };

        fn validate_component_type_for_dependency_type(
            dependency_type: DependencyType,
            component_type: AppComponentType,
        ) -> bool {
            match dependency_type {
                DependencyType::DynamicWasmRpc | DependencyType::StaticWasmRpc => {
                    match component_type {
                        AppComponentType::Durable | AppComponentType::Ephemeral => true,
                        AppComponentType::Library => false,
                    }
                }
                DependencyType::Wasm => match component_type {
                    AppComponentType::Durable | AppComponentType::Ephemeral => false,
                    AppComponentType::Library => true,
                },
            }
        }

        let component_names = {
            let app_ctx = self.ctx.app_context_lock().await;
            let app_ctx = app_ctx.some_or_err()?;
            app_ctx
                .application
                .component_names()
                .cloned()
                .collect::<Vec<_>>()
        };

        let component_name = {
            match component_name {
                Some(component_name) => {
                    if !component_names.contains(&component_name) {
                        log_error(format!(
                            "Component {} not found, available components: {}",
                            component_name.as_str().log_color_highlight(),
                            component_names
                                .iter()
                                .map(|name| name.as_str().log_color_highlight())
                                .join(", ")
                        ));
                        bail!(NonSuccessfulExit);
                    }
                    component_name
                }
                None => {
                    if component_names.is_empty() {
                        log_error(format!(
                            "No components found! Use the '{}' subcommand to create components.",
                            "component new".log_color_highlight()
                        ));
                        bail!(NonSuccessfulExit);
                    }

                    match Select::new(
                        "Select a component to which you want to add a new dependency:",
                        component_names.clone(),
                    )
                    .prompt()
                    .none_if_not_interactive_logged()?
                    {
                        Some(component_name) => component_name,
                        None => return Ok(None),
                    }
                }
            }
        };

        let component_type = component_type_by_name(&component_name).await?;

        log_action(
            "Selected",
            format!(
                "component {} with component type {}",
                component_name.as_str().log_color_highlight(),
                component_type.to_string().log_color_highlight()
            ),
        );

        let dependency_type = {
            let (offered_dependency_types, valid_dependency_types) = match component_type {
                AppComponentType::Durable | AppComponentType::Ephemeral => (
                    vec![DependencyType::DynamicWasmRpc, DependencyType::Wasm],
                    vec![
                        DependencyType::DynamicWasmRpc,
                        DependencyType::StaticWasmRpc,
                        DependencyType::Wasm,
                    ],
                ),
                AppComponentType::Library => {
                    (vec![DependencyType::Wasm], vec![DependencyType::Wasm])
                }
            };

            match dependency_type {
                Some(dependency_type) => {
                    if !valid_dependency_types.contains(&dependency_type) {
                        log_error(format!(
                            "The requested {} dependency type is not valid for {} component, valid dependency types: {}",
                            dependency_type.as_str().log_color_highlight(),
                            component_name.as_str().log_color_highlight(),
                            valid_dependency_types
                                .iter()
                                .map(|name| name.as_str().log_color_highlight())
                                .join(", ")
                        ));
                        bail!(NonSuccessfulExit);
                    }
                    dependency_type
                }
                None => {
                    match Select::new("Select dependency type:", offered_dependency_types)
                        .prompt()
                        .none_if_not_interactive_logged()?
                    {
                        Some(dependency_type) => dependency_type,
                        None => return Ok(None),
                    }
                }
            }
        };

        let target_component_name = match target_component_name {
            Some(target_component_name) => {
                if !component_names.contains(&target_component_name) {
                    log_error(format!(
                        "Target component {} not found, available components: {}",
                        target_component_name.as_str().log_color_highlight(),
                        component_names
                            .iter()
                            .map(|name| name.as_str().log_color_highlight())
                            .join(", ")
                    ));
                    bail!(NonSuccessfulExit);
                }

                let target_component_type = component_type_by_name(&target_component_name).await?;
                if !validate_component_type_for_dependency_type(
                    dependency_type,
                    target_component_type,
                ) {
                    log_error(
                        format!(
                            "The target component type {} is not compatible with the selected dependency type {}!",
                            target_component_type.to_string().log_color_highlight(),
                            dependency_type.as_str().log_color_highlight(),
                        )
                    );
                    logln("");
                    logln("Use a different target component or dependency type.");
                }

                target_component_name
            }
            None => {
                let target_component_names = {
                    let app_ctx = self.ctx.app_context_lock().await;
                    let app_ctx = app_ctx.some_or_err()?;
                    app_ctx
                        .application
                        .component_names()
                        .filter(|component_name| {
                            validate_component_type_for_dependency_type(
                                dependency_type,
                                app_ctx
                                    .application
                                    .component_properties(&component_name, self.ctx.build_profile())
                                    .component_type,
                            )
                        })
                        .cloned()
                        .collect::<Vec<_>>()
                };

                if target_component_names.is_empty() {
                    log_error(
                        "No target components are available for the selected dependency type!",
                    );
                    bail!(NonSuccessfulExit);
                }

                match Select::new(
                    "Select target dependency component:",
                    target_component_names,
                )
                .prompt()
                .none_if_not_interactive_logged()?
                {
                    Some(target_component_name) => target_component_name,
                    None => return Ok(None),
                }
            }
        };

        Ok(Some((
            component_name,
            target_component_name,
            dependency_type,
        )))
    }

    fn confirm<M: AsRef<str>>(&self, default: bool, message: M) -> anyhow::Result<bool> {
        const YES_FLAG_HINT: &str = "To automatically confirm such questions use the '--yes' flag.";

        if self.ctx.yes() {
            log_warn_action(
                "Auto confirming",
                format!("question: \"{}\"", message.as_ref().cyan()),
            );
            return Ok(true);
        }

        match Confirm::new(message.as_ref())
            .with_help_message(YES_FLAG_HINT)
            .with_default(default)
            .prompt()
        {
            Ok(result) => Ok(result),
            Err(error) => {
                if is_interactive_not_available_inquire_error(&error) {
                    log_warn("The current input device is not an interactive one,\ndefaulting to \"false\"");
                    Ok(false)
                } else {
                    Err(error.into())
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct OptionalUrl(Option<Url>);

impl Display for OptionalUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            None => Ok(()),
            Some(url) => write!(f, "{}", url),
        }
    }
}

impl FromStr for OptionalUrl {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            Ok(OptionalUrl(None))
        } else {
            Ok(OptionalUrl(Some(Url::from_str(s)?)))
        }
    }
}

trait InquireResultExtensions<T> {
    fn none_if_not_interactive(self) -> anyhow::Result<Option<T>>;

    fn none_if_not_interactive_logged(self) -> anyhow::Result<Option<T>>
    where
        Self: Sized,
    {
        self.none_if_not_interactive().inspect(|value| {
            if value.is_none() {
                logln("");
                log_warn("Detected non-interactive environment, stopping interactive wizard");
                logln("");
            }
        })
    }
}

impl<T> InquireResultExtensions<T> for InquireResult<T> {
    fn none_if_not_interactive(self) -> anyhow::Result<Option<T>> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(err) if is_interactive_not_available_inquire_error(&err) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

fn is_interactive_not_available_inquire_error(err: &InquireError) -> bool {
    match err {
        InquireError::NotTTY => true,
        InquireError::InvalidConfiguration(_) => false,
        InquireError::IO(_) => {
            // NOTE: we consider IO errors as "not interactive" in general, e.g. currently this case
            //       triggers if stdin is not available
            true
        }
        InquireError::OperationCanceled => false,
        InquireError::OperationInterrupted => false,
        InquireError::Custom(_) => false,
    }
}
