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

use crate::command::GolemCliGlobalFlags;
use crate::config::{
    BuildProfileName, ClientConfig, HttpClientConfig, NamedProfile, Profile, ProfileName,
};
use crate::model::app_ext::GolemComponentExtensions;
use anyhow::anyhow;
use golem_client::api::ApiDefinitionClientLive as ApiDefinitionClientOss;
use golem_client::api::ApiDeploymentClientLive as ApiDeploymentClientOss;
use golem_client::api::ApiSecurityClientLive as ApiSecurityClientOss;
use golem_client::api::ComponentClientLive as ComponentClientOss;
use golem_client::api::PluginClientLive as PluginClientOss;
use golem_client::api::WorkerClientLive as WorkerClientOss;
use golem_client::Context as ContextOss;
use golem_cloud_client::api::AccountClientLive as AccountClientCloud;
use golem_cloud_client::api::AccountSummaryClientLive as AccountSummaryClientCloud;
use golem_cloud_client::api::ApiCertificateClientLive as ApiCertificateClientCloud;
use golem_cloud_client::api::ApiDefinitionClientLive as ApiDefinitionClientCloud;
use golem_cloud_client::api::ApiDeploymentClientLive as ApiDeploymentClientCloud;
use golem_cloud_client::api::ApiDomainClientLive as ApiDomainClientCloud;
use golem_cloud_client::api::ApiSecurityClientLive as ApiSecurityClientCloud;
use golem_cloud_client::api::ComponentClientLive as ComponentClientCloud;
use golem_cloud_client::api::GrantClientLive as GrantClientCloud;
use golem_cloud_client::api::LimitsClientLive as LimitsClientCloud;
use golem_cloud_client::api::LoginClientLive as LoginClientCloud;
use golem_cloud_client::api::PluginClientLive as PluginClientCloud;
use golem_cloud_client::api::ProjectClientLive as ProjectClientCloud;
use golem_cloud_client::api::ProjectGrantClientLive as ProjectGrantClientCloud;
use golem_cloud_client::api::ProjectPolicyClientLive as ProjectPolicyClientCloud;
use golem_cloud_client::api::TokenClientLive as TokenClientCloud;
use golem_cloud_client::api::WorkerClientLive as WorkerClientCloud;
use golem_cloud_client::{Context as ContextCloud, Security};
use golem_examples::model::{ComposableAppGroupName, GuestLanguage};
use golem_examples::ComposableAppExample;
use golem_wasm_rpc_stubgen::commands::app::{
    ApplicationContext, ApplicationSourceMode, ComponentSelectMode,
};
use golem_wasm_rpc_stubgen::log::{set_log_output, Output};
use golem_wasm_rpc_stubgen::model::app::AppBuildStep;
use golem_wasm_rpc_stubgen::stub::WasmRpcOverride;
use std::collections::{BTreeMap, HashSet};
use std::marker::PhantomData;
use std::path::PathBuf;
use tracing::debug;

pub struct Context {
    profile_name: ProfileName,
    profile: Profile,
    build_profile: Option<BuildProfileName>,
    app_manifest_path: Option<PathBuf>,
    wasm_rpc_override: WasmRpcOverride,
    clients: tokio::sync::OnceCell<Clients>,
    application_context: tokio::sync::OnceCell<ApplicationContext<GolemComponentExtensions>>,
    templates: std::cell::OnceCell<
        BTreeMap<GuestLanguage, BTreeMap<ComposableAppGroupName, ComposableAppExample>>,
    >,
    missing_templates: BTreeMap<ComposableAppGroupName, ComposableAppExample>,
    component_select_mode: ComponentSelectMode,
    component_select_mode_was_set: bool,
    skip_up_to_date_checks: bool,
    skip_up_to_date_checks_was_set: bool,
    build_steps_filter: HashSet<AppBuildStep>,
    build_steps_filter_was_set: bool,
}

impl Context {
    pub fn new(global_flags: &GolemCliGlobalFlags, profile: NamedProfile) -> Self {
        set_log_output(Output::Stderr);

        Self {
            profile_name: profile.name,
            profile: profile.profile,
            build_profile: global_flags.build_profile.clone(),
            app_manifest_path: global_flags.app_manifest_path.clone(),
            wasm_rpc_override: WasmRpcOverride {
                wasm_rpc_path_override: global_flags.wasm_rpc_path.clone(),
                wasm_rpc_version_override: global_flags.wasm_rpc_version.clone(),
            },
            clients: tokio::sync::OnceCell::new(),
            application_context: tokio::sync::OnceCell::new(),
            templates: std::cell::OnceCell::new(),
            missing_templates: BTreeMap::new(),
            component_select_mode: ComponentSelectMode::CurrentDir,
            component_select_mode_was_set: false,
            skip_up_to_date_checks: false,
            skip_up_to_date_checks_was_set: false,
            build_steps_filter: HashSet::new(),
            build_steps_filter_was_set: false,
        }
    }

    pub async fn clients(&self) -> anyhow::Result<&Clients> {
        self.clients
            .get_or_try_init(|| async { Clients::new((&self.profile).into()).await })
            .await
    }

    pub async fn golem_clients(&self) -> anyhow::Result<&GolemClients> {
        Ok(&self.clients().await?.golem)
    }

    pub async fn application_context(
        &self,
    ) -> anyhow::Result<&ApplicationContext<GolemComponentExtensions>> {
        self.application_context
            .get_or_try_init(|| async {
                let config = golem_wasm_rpc_stubgen::commands::app::Config {
                    app_source_mode: {
                        match &self.app_manifest_path {
                            Some(path) => ApplicationSourceMode::Explicit(vec![path.clone()]),
                            None => ApplicationSourceMode::Automatic,
                        }
                    },
                    component_select_mode: self.component_select_mode.clone(),
                    skip_up_to_date_checks: self.skip_up_to_date_checks,
                    profile: self.build_profile.as_ref().map(|p| p.to_string().into()),
                    offline: false, // TODO:
                    extensions: PhantomData::<GolemComponentExtensions>,
                    steps_filter: self.build_steps_filter.clone(),
                    wasm_rpc_override: self.wasm_rpc_override.clone(),
                };

                debug!(config = ?config, "Initializing application context");

                Ok(ApplicationContext::new(config)?)
            })
            .await
    }

    pub async fn application_context_mut(
        &mut self,
    ) -> anyhow::Result<&mut ApplicationContext<GolemComponentExtensions>> {
        let _ = self.application_context().await?;
        Ok(self.application_context.get_mut().unwrap())
    }

    fn set_app_ctx_config<T>(
        &mut self,
        name: &str,
        get_value_mut: fn(&mut Context) -> &mut T,
        get_was_set_mut: fn(&mut Context) -> &mut bool,
        value: T,
    ) {
        if *get_was_set_mut(self) {
            panic!("{} can be set only once, was already set", name);
        }
        if self.application_context.get().is_some() {
            panic!("cannot change {} after application context init", name);
        }
        *get_value_mut(self) = value;
        *get_was_set_mut(self) = true;
    }

    pub fn set_component_select_mode(&mut self, mode: ComponentSelectMode) {
        self.set_app_ctx_config(
            "component_select_mode",
            |ctx| &mut ctx.component_select_mode,
            |ctx| &mut ctx.component_select_mode_was_set,
            mode,
        );
    }

    pub fn set_skip_up_to_date_checks(&mut self, skip: bool) {
        self.set_app_ctx_config(
            "skip_up_to_date_checks",
            |ctx| &mut ctx.skip_up_to_date_checks,
            |ctx| &mut ctx.skip_up_to_date_checks_was_set,
            skip,
        )
    }

    pub fn set_steps_filter(&mut self, steps_filter: HashSet<AppBuildStep>) {
        self.set_app_ctx_config(
            "steps_filter",
            |ctx| &mut ctx.build_steps_filter,
            |ctx| &mut ctx.build_steps_filter_was_set,
            steps_filter,
        );
    }

    pub fn templates(
        &self,
    ) -> &BTreeMap<GuestLanguage, BTreeMap<ComposableAppGroupName, ComposableAppExample>> {
        self.templates
            .get_or_init(|| golem_examples::all_composable_app_examples())
    }

    fn language_templates(
        &self,
        language: GuestLanguage,
    ) -> &BTreeMap<ComposableAppGroupName, ComposableAppExample> {
        self.templates()
            .get(&language)
            .unwrap_or(&self.missing_templates)
    }
}

// TODO: add healthcheck clients
pub struct Clients {
    pub golem: GolemClients,
    pub file_download_http_client: reqwest::Client,
}

impl Clients {
    pub async fn new(config: ClientConfig) -> anyhow::Result<Self> {
        let service_http_client = new_reqwest_client(&config.service_http_client_config)?;
        let file_download_http_client =
            new_reqwest_client(&config.file_download_http_client_config)?;

        match &config.cloud_url {
            Some(cloud_url) => {
                // TODO:
                let security_token = Security::Bearer("dummy-token".to_string());

                let component_context = || ContextCloud {
                    client: service_http_client.clone(),
                    base_url: config.component_url.clone(),
                    security_token: security_token.clone(),
                };

                let worker_context = || ContextCloud {
                    client: service_http_client.clone(),
                    base_url: config.worker_url.clone(),
                    security_token: security_token.clone(),
                };

                let cloud_context = || ContextCloud {
                    client: service_http_client.clone(),
                    base_url: cloud_url.clone(),
                    security_token: security_token.clone(),
                };

                let login_context = || ContextCloud {
                    client: service_http_client.clone(),
                    base_url: cloud_url.clone(),
                    security_token: Security::Empty,
                };

                Ok(Clients {
                    golem: GolemClients::Cloud(GolemClientsCloud {
                        account: AccountClientCloud {
                            context: cloud_context(),
                        },
                        account_summary: AccountSummaryClientCloud {
                            context: worker_context(),
                        },
                        api_certificate: ApiCertificateClientCloud {
                            context: worker_context(),
                        },
                        api_definition: ApiDefinitionClientCloud {
                            context: worker_context(),
                        },
                        api_deployment: ApiDeploymentClientCloud {
                            context: worker_context(),
                        },
                        api_domain: ApiDomainClientCloud {
                            context: worker_context(),
                        },
                        api_security: ApiSecurityClientCloud {
                            context: worker_context(),
                        },
                        component: ComponentClientCloud {
                            context: component_context(),
                        },
                        grant: GrantClientCloud {
                            context: cloud_context(),
                        },
                        limits: LimitsClientCloud {
                            context: worker_context(),
                        },
                        login: LoginClientCloud {
                            context: login_context(),
                        },
                        plugin: PluginClientCloud {
                            context: component_context(),
                        },
                        project: ProjectClientCloud {
                            context: cloud_context(),
                        },
                        project_grant: ProjectGrantClientCloud {
                            context: cloud_context(),
                        },
                        project_policy: ProjectPolicyClientCloud {
                            context: cloud_context(),
                        },
                        token: TokenClientCloud {
                            context: cloud_context(),
                        },
                        worker: WorkerClientCloud {
                            context: worker_context(),
                        },
                    }),
                    file_download_http_client,
                })
            }
            None => {
                let component_context = || ContextOss {
                    client: service_http_client.clone(),
                    base_url: config.component_url.clone(),
                };

                let worker_context = || ContextOss {
                    client: service_http_client.clone(),
                    base_url: config.worker_url.clone(),
                };

                Ok(Clients {
                    golem: GolemClients::Oss(GolemClientsOss {
                        api_definition: ApiDefinitionClientOss {
                            context: worker_context(),
                        },
                        api_deployment: ApiDeploymentClientOss {
                            context: worker_context(),
                        },
                        api_security: ApiSecurityClientOss {
                            context: worker_context(),
                        },
                        component: ComponentClientOss {
                            context: component_context(),
                        },
                        plugin: PluginClientOss {
                            context: component_context(),
                        },
                        worker: WorkerClientOss {
                            context: worker_context(),
                        },
                    }),
                    file_download_http_client,
                })
            }
        }
    }
}

pub enum GolemClients {
    Oss(GolemClientsOss),
    Cloud(GolemClientsCloud),
}

pub struct GolemClientsOss {
    pub api_definition: ApiDefinitionClientOss,
    pub api_deployment: ApiDeploymentClientOss,
    pub api_security: ApiSecurityClientOss,
    pub component: ComponentClientOss,
    pub plugin: PluginClientOss,
    pub worker: WorkerClientOss,
}

pub struct GolemClientsCloud {
    pub account: AccountClientCloud,
    pub account_summary: AccountSummaryClientCloud,
    pub api_certificate: ApiCertificateClientCloud,
    pub api_definition: ApiDefinitionClientCloud,
    pub api_deployment: ApiDeploymentClientCloud,
    pub api_domain: ApiDomainClientCloud,
    pub api_security: ApiSecurityClientCloud,
    pub component: ComponentClientCloud,
    pub grant: GrantClientCloud,
    pub limits: LimitsClientCloud,
    pub login: LoginClientCloud,
    pub plugin: PluginClientCloud,
    pub project: ProjectClientCloud,
    pub project_grant: ProjectGrantClientCloud,
    pub project_policy: ProjectPolicyClientCloud,
    pub token: TokenClientCloud,
    pub worker: WorkerClientCloud,
}

fn new_reqwest_client(config: &HttpClientConfig) -> anyhow::Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder();

    if config.allow_insecure {
        builder = builder.danger_accept_invalid_certs(true);
    }

    if let Some(timeout) = config.timeout {
        builder = builder.timeout(timeout);
    }
    if let Some(connect_timeout) = config.connect_timeout {
        builder = builder.connect_timeout(connect_timeout);
    }
    if let Some(read_timeout) = config.read_timeout {
        builder = builder.read_timeout(read_timeout);
    }

    Ok(builder.connection_verbose(true).build()?)
}
