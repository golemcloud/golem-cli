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

use crate::app::context::ApplicationContext;
use crate::auth::{Auth, CloudAuthentication};
use crate::cloud::{AccountId, CloudAuthenticationConfig};
use crate::command::shared_args::UpdateOrRedeployArgs;
use crate::command::GolemCliGlobalFlags;
use crate::config::{
    ClientConfig, CloudProfile, Config, HttpClientConfig, NamedProfile, OssProfile, Profile,
    ProfileConfig, ProfileKind, ProfileName,
};
use crate::error::{ContextInitHintError, HintError};
use crate::log::{log_action, set_log_output, LogColorize, LogOutput, Output};
use crate::model::app::{AppBuildStep, ApplicationSourceMode};
use crate::model::app::{ApplicationConfig, BuildProfileName as AppBuildProfileName};
use crate::model::{app_raw, Format, HasFormatConfig, ProjectReference};
use crate::wasm_rpc_stubgen::stub::RustDependencyOverride;
use anyhow::{anyhow, bail, Context as AnyhowContext};
use futures_util::future::BoxFuture;
use golem_client::api::ApiDefinitionClientLive as ApiDefinitionClientOss;
use golem_client::api::ApiDeploymentClientLive as ApiDeploymentClientOss;
use golem_client::api::ApiSecurityClientLive as ApiSecurityClientOss;
use golem_client::api::ComponentClientLive as ComponentClientOss;
use golem_client::api::HealthCheckClientLive as HealthCheckClientOss;
use golem_client::api::PluginClientLive as PluginClientOss;
use golem_client::api::WorkerClientLive as WorkerClientOss;
use golem_client::Context as ContextOss;
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
use golem_cloud_client::api::{AccountClientLive as AccountClientCloud, LoginClientLive};
use golem_cloud_client::{Context as ContextCloud, Security};
use golem_rib_repl::ReplDependencies;
use golem_templates::model::{ComposableAppGroupName, GuestLanguage};
use golem_templates::ComposableAppTemplate;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use tracing::debug;
use url::Url;
use uuid::Uuid;

// Context is responsible for storing the CLI state,
// but NOT responsible for producing CLI output, those should be part of the CommandHandler(s)
pub struct Context {
    // Readonly
    config_dir: PathBuf,
    format: Format,
    local_server_auto_start: bool,
    update_or_redeploy: UpdateOrRedeployArgs,
    profile_name: ProfileName,
    profile_kind: ProfileKind,
    profile: Profile,
    available_profile_names: BTreeSet<ProfileName>,
    custom_cloud_profile_name: Option<ProfileName>,
    app_context_config: ApplicationContextConfig,
    http_batch_size: u64,
    auth_token_override: Option<Uuid>,
    project: Option<ProjectReference>,
    client_config: ClientConfig,
    yes: bool,
    #[allow(unused)]
    start_local_server: Box<dyn Fn() -> BoxFuture<'static, anyhow::Result<()>> + Send + Sync>,

    // Lazy initialized
    clients: tokio::sync::OnceCell<Clients>,
    templates: std::sync::OnceLock<
        BTreeMap<GuestLanguage, BTreeMap<ComposableAppGroupName, ComposableAppTemplate>>,
    >,

    // Directly mutable
    app_context_state: tokio::sync::RwLock<ApplicationContextState>,
    rib_repl_state: tokio::sync::RwLock<RibReplState>,
}

impl Context {
    pub async fn new(
        global_flags: GolemCliGlobalFlags,
        start_local_server_yes: Arc<tokio::sync::RwLock<bool>>,
        start_local_server: Box<dyn Fn() -> BoxFuture<'static, anyhow::Result<()>> + Send + Sync>,
    ) -> anyhow::Result<Self> {
        let format = global_flags.format;
        let http_batch_size = global_flags.http_batch_size;
        let auth_token = global_flags.auth_token;
        let config_dir = global_flags.config_dir();
        let custom_cloud_profile_name = global_flags.custom_global_cloud_profile.clone();
        let local_server_auto_start = global_flags.local_server_auto_start;

        let mut yes = global_flags.yes;
        let mut update_or_redeploy = UpdateOrRedeployArgs::none();

        let mut app_context_config = ApplicationContextConfig::new(global_flags);

        let (manifest_profiles, app_source_mode) =
            ApplicationContext::preload_sources_and_get_profiles(
                app_context_config.app_source_mode(),
            )?;
        let manifest_profiles = manifest_profiles.unwrap_or_default();

        let (available_profile_names, profile, manifest_profile) = load_merged_profiles(
            &config_dir,
            custom_cloud_profile_name.as_ref(),
            app_context_config.requested_profile_name.as_ref(),
            manifest_profiles,
        )?;

        debug!(profile_name=%profile.name, manifest_profile=?manifest_profile, "Loaded profiles");

        if let Some(manifest_profile) = &manifest_profile {
            if app_context_config.build_profile.is_none() {
                app_context_config.build_profile = manifest_profile
                    .build_profile
                    .as_ref()
                    .map(|build_profile| build_profile.as_str().into())
            }

            if manifest_profile.auto_confirm == Some(true) {
                yes = true;
                *start_local_server_yes.write().await = true;
            }

            if manifest_profile.redeploy_workers == Some(true) {
                update_or_redeploy.redeploy_workers = true;
            }

            if manifest_profile.redeploy_http_api == Some(true) {
                update_or_redeploy.redeploy_http_api = true;
            }

            if manifest_profile.redeploy_all == Some(true) {
                update_or_redeploy.redeploy_all = true;
            }
        }

        let project = match manifest_profile.as_ref().and_then(|m| m.project.as_ref()) {
            Some(project) => Some(
                ProjectReference::from_str(project.as_str())
                    .map_err(|err| anyhow!("{}", err))
                    .with_context(|| {
                        anyhow!(
                            "Failed to parse project for manifest profile {}",
                            profile.name.0.log_color_highlight()
                        )
                    })?,
            ),
            None => None,
        };

        let format = format.unwrap_or_else(|| profile.profile.format().unwrap_or(Format::Text));
        let log_output = match format {
            Format::Json => Output::Stderr,
            Format::Yaml => Output::Stderr,
            Format::Text => Output::Stdout,
        };
        set_log_output(log_output);

        log_action(
            "Selected",
            format!(
                "profile: {}{}",
                profile.name.0.log_color_highlight(),
                project
                    .as_ref()
                    .map(|project| format!(
                        ", project: {}",
                        project.to_string().log_color_highlight()
                    ))
                    .unwrap_or_else(|| "".to_string())
            ),
        );

        let client_config = ClientConfig::from(&profile.profile);

        Ok(Self {
            config_dir,
            format,
            local_server_auto_start,
            update_or_redeploy,
            profile_name: profile.name,
            profile_kind: profile.profile.kind(),
            profile: profile.profile,
            available_profile_names,
            custom_cloud_profile_name,
            app_context_config,
            http_batch_size: http_batch_size.unwrap_or(50),
            auth_token_override: auth_token,
            project,
            yes,
            start_local_server,
            client_config,
            clients: tokio::sync::OnceCell::new(),
            templates: std::sync::OnceLock::new(),
            app_context_state: tokio::sync::RwLock::new(ApplicationContextState::new(
                app_source_mode,
            )),
            rib_repl_state: tokio::sync::RwLock::default(),
        })
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub async fn rib_repl_history_file(&self) -> anyhow::Result<PathBuf> {
        let app_ctx = self.app_context_lock().await;
        let history_file = match app_ctx.opt()? {
            Some(app_ctx) => app_ctx.application.rib_repl_history_file().to_path_buf(),
            None => self.config_dir.join(".rib_repl_history"),
        };
        debug!(
            history_file = %history_file.display(),
            "Selected Rib REPL history file"
        );
        Ok(history_file)
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub fn yes(&self) -> bool {
        self.yes
    }

    pub fn update_or_redeploy(&self) -> &UpdateOrRedeployArgs {
        &self.update_or_redeploy
    }

    pub async fn silence_app_context_init(&self) {
        let mut state = self.app_context_state.write().await;
        state.silent_init = true;
    }

    pub fn profile_kind(&self) -> ProfileKind {
        self.profile_kind
    }

    pub fn profile_name(&self) -> &ProfileName {
        &self.profile_name
    }

    pub fn available_profile_names(&self) -> &BTreeSet<ProfileName> {
        &self.available_profile_names
    }

    pub fn build_profile(&self) -> Option<&AppBuildProfileName> {
        self.app_context_config.build_profile.as_ref()
    }

    pub fn profile_project(&self) -> Option<&ProjectReference> {
        self.project.as_ref()
    }

    pub fn http_batch_size(&self) -> u64 {
        self.http_batch_size
    }

    pub async fn clients(&self) -> anyhow::Result<&Clients> {
        self.clients
            .get_or_try_init(|| async {
                let clients = Clients::new(
                    self.client_config.clone(),
                    self.auth_token_override,
                    {
                        if self.profile_name.is_builtin_cloud() {
                            self.custom_cloud_profile_name
                                .as_ref()
                                .unwrap_or(&self.profile_name)
                        } else {
                            &self.profile_name
                        }
                    },
                    match &self.profile {
                        Profile::Golem(_) => None,
                        Profile::GolemCloud(profile) => profile.auth.as_ref(),
                    },
                    self.config_dir(),
                )
                .await?;

                if self.local_server_auto_start {
                    self.start_local_server_if_needed(&clients).await?;
                }

                Ok(clients)
            })
            .await
    }

    #[cfg(feature = "server-commands")]
    async fn start_local_server_if_needed(&self, clients: &Clients) -> anyhow::Result<()> {
        if !self.profile_name.is_builtin_local() {
            return Ok(());
        };

        let GolemClients::Oss(clients) = &clients.golem else {
            return Ok(());
        };

        // NOTE: explicitly calling the trait method to avoid unused imports when compiling with
        //       default features
        if golem_client::api::HealthCheckClient::healthcheck(&clients.component_healthcheck)
            .await
            .is_ok()
        {
            return Ok(());
        }

        (self.start_local_server)().await?;

        Ok(())
    }

    #[cfg(not(feature = "server-commands"))]
    async fn start_local_server_if_needed(&self, _clients: &Clients) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn golem_clients(&self) -> anyhow::Result<&GolemClients> {
        Ok(&self.clients().await?.golem)
    }

    pub async fn file_download_client(&self) -> anyhow::Result<reqwest::Client> {
        Ok(self.clients().await?.file_download.clone())
    }

    pub async fn golem_clients_cloud(&self) -> anyhow::Result<&GolemClientsCloud> {
        match &self.clients().await?.golem {
            GolemClients::Oss(_) => Err(anyhow!(HintError::ExpectedCloudProfile)),
            GolemClients::Cloud(clients) => Ok(clients),
        }
    }

    pub fn worker_service_url(&self) -> &Url {
        &self.client_config.worker_url
    }

    pub fn allow_insecure(&self) -> bool {
        self.client_config.service_http_client_config.allow_insecure
    }

    pub async fn auth_token(&self) -> anyhow::Result<Option<String>> {
        match self.golem_clients().await? {
            GolemClients::Oss(_) => Ok(None),
            GolemClients::Cloud(clients) => Ok(Some(clients.auth_token())),
        }
    }

    pub async fn app_context_lock(
        &self,
    ) -> tokio::sync::RwLockReadGuard<'_, ApplicationContextState> {
        {
            let state = self.app_context_state.read().await;
            if state.app_context.is_some() {
                return state;
            }
        }

        {
            let _init = self.app_context_lock_mut().await;
        }

        self.app_context_state.read().await
    }

    pub async fn app_context_lock_mut(
        &self,
    ) -> tokio::sync::RwLockWriteGuard<'_, ApplicationContextState> {
        let mut state = self.app_context_state.write().await;
        state.init(&self.available_profile_names, &self.app_context_config);
        state
    }

    pub async fn unload_app_context(&self) {
        let mut state = self.app_context_state.write().await;
        *state = ApplicationContextState::new(self.app_context_config.app_source_mode());
    }

    async fn set_app_ctx_init_config<T>(
        &self,
        name: &str,
        value_mut: fn(&mut ApplicationContextState) -> &mut T,
        was_set_mut: fn(&mut ApplicationContextState) -> &mut bool,
        value: T,
    ) {
        let mut state = self.app_context_state.write().await;
        if *was_set_mut(&mut state) {
            panic!("{} can be set only once, was already set", name);
        }
        if state.app_context.is_some() {
            panic!("cannot change {} after application context init", name);
        }
        *value_mut(&mut state) = value;
        *was_set_mut(&mut state) = true;
    }

    pub async fn set_skip_up_to_date_checks(&self, skip: bool) {
        self.set_app_ctx_init_config(
            "skip_up_to_date_checks",
            |ctx| &mut ctx.skip_up_to_date_checks,
            |ctx| &mut ctx.skip_up_to_date_checks_was_set,
            skip,
        )
        .await
    }

    pub async fn set_steps_filter(&self, steps_filter: HashSet<AppBuildStep>) {
        self.set_app_ctx_init_config(
            "steps_filter",
            |ctx| &mut ctx.build_steps_filter,
            |ctx| &mut ctx.build_steps_filter_was_set,
            steps_filter,
        )
        .await;
    }

    pub async fn task_result_marker_dir(&self) -> anyhow::Result<PathBuf> {
        let app_ctx = self.app_context_lock().await;
        let app_ctx = app_ctx.some_or_err()?;
        Ok(app_ctx.application.task_result_marker_dir())
    }

    pub async fn set_rib_repl_dependencies(&self, dependencies: ReplDependencies) {
        let mut rib_repl_state = self.rib_repl_state.write().await;
        rib_repl_state.dependencies = dependencies;
    }

    pub async fn get_rib_repl_dependencies(&self) -> ReplDependencies {
        let rib_repl_state = self.rib_repl_state.read().await;
        ReplDependencies {
            component_dependencies: rib_repl_state.dependencies.component_dependencies.clone(),
        }
    }

    pub fn templates(
        &self,
    ) -> &BTreeMap<GuestLanguage, BTreeMap<ComposableAppGroupName, ComposableAppTemplate>> {
        self.templates
            .get_or_init(golem_templates::all_composable_app_templates)
    }
}

pub struct Clients {
    pub golem: GolemClients,
    pub file_download: reqwest::Client,
}

impl Clients {
    pub async fn new(
        config: ClientConfig,
        token_override: Option<Uuid>,
        profile_name: &ProfileName,
        auth_config: Option<&CloudAuthenticationConfig>,
        config_dir: &Path,
    ) -> anyhow::Result<Self> {
        let healthcheck_http_client = new_reqwest_client(&config.health_check_http_client_config)?;
        let local_healthcheck_http_client =
            new_reqwest_client(&config.local_health_check_http_client_config)?;
        let service_http_client = new_reqwest_client(&config.service_http_client_config)?;
        let invoke_http_client = new_reqwest_client(&config.invoke_http_client_config)?;
        let file_download_http_client =
            new_reqwest_client(&config.file_download_http_client_config)?;

        match &config.cloud_url {
            Some(cloud_url) => {
                let auth = Auth::new(LoginClientLive {
                    context: ContextCloud {
                        client: service_http_client.clone(),
                        base_url: cloud_url.clone(),
                        security_token: Security::Empty,
                    },
                });

                let authentication = auth
                    .authenticate(token_override, auth_config, config_dir, profile_name)
                    .await?;
                let security_token = Security::Bearer(authentication.0.secret.value.to_string());

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

                let worker_invoke_context = || ContextCloud {
                    client: invoke_http_client.clone(),
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
                    security_token: security_token.clone(),
                };

                Ok(Clients {
                    golem: GolemClients::Cloud(GolemClientsCloud {
                        authentication,
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
                        worker_invoke: WorkerClientCloud {
                            context: worker_invoke_context(),
                        },
                    }),
                    file_download: file_download_http_client,
                })
            }
            None => {
                let component_context = || ContextOss {
                    client: service_http_client.clone(),
                    base_url: config.component_url.clone(),
                };

                let component_healthcheck_context = || ContextOss {
                    client: {
                        if profile_name.is_builtin_local() {
                            local_healthcheck_http_client.clone()
                        } else {
                            healthcheck_http_client.clone()
                        }
                    },
                    base_url: config.component_url.clone(),
                };

                let worker_context = || ContextOss {
                    client: service_http_client.clone(),
                    base_url: config.worker_url.clone(),
                };

                let worker_invoke_context = || ContextOss {
                    client: invoke_http_client.clone(),
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
                        component_healthcheck: HealthCheckClientOss {
                            context: component_healthcheck_context(),
                        },
                        plugin: PluginClientOss {
                            context: component_context(),
                        },
                        worker: WorkerClientOss {
                            context: worker_context(),
                        },
                        worker_invoke: WorkerClientOss {
                            context: worker_invoke_context(),
                        },
                    }),
                    file_download: file_download_http_client,
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
    pub component_healthcheck: HealthCheckClientOss,
    pub plugin: PluginClientOss,
    pub worker: WorkerClientOss,
    pub worker_invoke: WorkerClientOss,
}

pub struct GolemClientsCloud {
    authentication: CloudAuthentication,

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
    pub worker_invoke: WorkerClientCloud,
}

impl GolemClientsCloud {
    pub fn account_id(&self) -> AccountId {
        self.authentication.account_id()
    }

    pub fn auth_token(&self) -> String {
        self.authentication.0.secret.value.to_string()
    }
}

struct ApplicationContextConfig {
    requested_profile_name: Option<ProfileName>,
    build_profile: Option<AppBuildProfileName>,
    app_manifest_path: Option<PathBuf>,
    disable_app_manifest_discovery: bool,
    golem_rust_override: RustDependencyOverride,
    wasm_rpc_client_build_offline: bool,
}

impl ApplicationContextConfig {
    pub fn new(global_flags: GolemCliGlobalFlags) -> Self {
        Self {
            requested_profile_name: {
                if global_flags.local {
                    Some(ProfileName::local())
                } else if global_flags.cloud {
                    Some(ProfileName::cloud())
                } else {
                    global_flags.profile.clone()
                }
            },
            build_profile: global_flags.build_profile.map(|bp| bp.0.into()),
            app_manifest_path: global_flags.app_manifest_path,
            disable_app_manifest_discovery: global_flags.disable_app_manifest_discovery,
            golem_rust_override: RustDependencyOverride {
                path_override: global_flags.golem_rust_path,
                version_override: global_flags.golem_rust_version,
            },
            wasm_rpc_client_build_offline: global_flags.wasm_rpc_offline,
        }
    }

    pub fn app_source_mode(&self) -> ApplicationSourceMode {
        if self.disable_app_manifest_discovery {
            ApplicationSourceMode::None
        } else {
            match &self.app_manifest_path {
                Some(root_manifest) if !self.disable_app_manifest_discovery => {
                    ApplicationSourceMode::ByRootManifest(root_manifest.clone())
                }
                _ => ApplicationSourceMode::Automatic,
            }
        }
    }
}

#[derive()]
pub struct ApplicationContextState {
    app_source_mode: Option<ApplicationSourceMode>,
    pub silent_init: bool,
    pub skip_up_to_date_checks: bool,
    skip_up_to_date_checks_was_set: bool,
    pub build_steps_filter: HashSet<AppBuildStep>,
    build_steps_filter_was_set: bool,

    app_context: Option<Result<Option<ApplicationContext>, Arc<anyhow::Error>>>,
}

impl ApplicationContextState {
    pub fn new(source_mode: ApplicationSourceMode) -> Self {
        Self {
            app_source_mode: Some(source_mode),
            silent_init: false,
            skip_up_to_date_checks: false,
            skip_up_to_date_checks_was_set: false,
            build_steps_filter: HashSet::new(),
            build_steps_filter_was_set: false,
            app_context: None,
        }
    }

    fn init(
        &mut self,
        available_profile_names: &BTreeSet<ProfileName>,
        config: &ApplicationContextConfig,
    ) {
        if self.app_context.is_some() {
            return;
        }

        let _log_output = self
            .silent_init
            .then(|| LogOutput::new(Output::TracingDebug));

        let app_config = ApplicationConfig {
            skip_up_to_date_checks: self.skip_up_to_date_checks,
            build_profile: config.build_profile.as_ref().map(|p| p.to_string().into()),
            offline: config.wasm_rpc_client_build_offline,
            steps_filter: self.build_steps_filter.clone(),
            golem_rust_override: config.golem_rust_override.clone(),
        };

        debug!(app_config = ?app_config, "Initializing application context");

        self.app_context = Some(
            ApplicationContext::new(
                available_profile_names,
                self.app_source_mode
                    .take()
                    .expect("ApplicationContextState.app_source_mode is not set"),
                app_config,
            )
            .map_err(Arc::new),
        )
    }

    pub fn opt(&self) -> anyhow::Result<Option<&ApplicationContext>> {
        match &self.app_context {
            Some(Ok(None)) => Ok(None),
            Some(Ok(Some(app_ctx))) => Ok(Some(app_ctx)),
            Some(Err(err)) => Err(anyhow!(err.clone())),
            None => unreachable!("Uninitialized application context"),
        }
    }

    pub fn opt_mut(&mut self) -> anyhow::Result<Option<&mut ApplicationContext>> {
        match &mut self.app_context {
            Some(Ok(None)) => Ok(None),
            Some(Ok(Some(app_ctx))) => Ok(Some(app_ctx)),
            Some(Err(err)) => Err(anyhow!(err.clone())),
            None => unreachable!("Uninitialized application context"),
        }
    }

    pub fn some_or_err(&self) -> anyhow::Result<&ApplicationContext> {
        match &self.app_context {
            Some(Ok(None)) => Err(anyhow!(HintError::NoApplicationManifestFound)),
            Some(Ok(Some(app_ctx))) => Ok(app_ctx),
            Some(Err(err)) => Err(anyhow!(err.clone())),
            None => unreachable!("Uninitialized application context"),
        }
    }

    pub fn some_or_err_mut(&mut self) -> anyhow::Result<&mut ApplicationContext> {
        match &mut self.app_context {
            Some(Ok(None)) => Err(anyhow!(HintError::NoApplicationManifestFound)),
            Some(Ok(Some(app_ctx))) => Ok(app_ctx),
            Some(Err(err)) => Err(anyhow!(err.clone())),
            None => unreachable!("Uninitialized application context"),
        }
    }
}

pub struct RibReplState {
    dependencies: ReplDependencies,
}

impl Default for RibReplState {
    fn default() -> Self {
        Self {
            dependencies: ReplDependencies {
                component_dependencies: vec![],
            },
        }
    }
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

/// Finds the requested or the default profile in the global CLI config
/// and in the application manifest. The global config gets overrides applied from
/// the manifest profile.
///
/// NOTE: Both of the profiles are returned, because currently the set of properties
///       are different. Eventually they should converge, but that will need breaking
///       config changes and migration.
fn load_merged_profiles(
    config_dir: &Path,
    custom_cloud_profile_name: Option<&ProfileName>,
    profile_name: Option<&ProfileName>,
    manifest_profiles: BTreeMap<ProfileName, app_raw::Profile>,
) -> anyhow::Result<(
    BTreeSet<ProfileName>,
    NamedProfile,
    Option<app_raw::Profile>,
)> {
    let cloud_profile_name = custom_cloud_profile_name
        .cloned()
        .unwrap_or_else(ProfileName::cloud);

    let mut available_profile_names = BTreeSet::new();

    let mut config = Config::from_dir(config_dir)?;
    available_profile_names.extend(config.profiles.keys().cloned());
    available_profile_names.extend(manifest_profiles.keys().cloned());

    // Use the requested profile name or the manifest default profile if none was requested
    // and there is a manifest default one
    let profile_name = match profile_name {
        Some(profile_name) => Some(profile_name),
        None => manifest_profiles
            .iter()
            .find_map(|(profile_name, profile)| {
                (profile.default == Some(true)).then_some(profile_name)
            }),
    };

    let global_profile = match profile_name {
        Some(profile_name) => config
            .profiles
            .remove({
                if profile_name.is_builtin_cloud() {
                    &cloud_profile_name
                } else {
                    profile_name
                }
            })
            .map(|profile| NamedProfile {
                name: profile_name.clone(),
                profile,
            }),
        None => Some(Config::get_default_profile(config_dir)?),
    };

    // If we did not find a global (maybe default) profile then switch back to the previously
    // calculated requested or manifest default profile.
    let profile_name = global_profile
        .as_ref()
        .map(|profile| Some(profile.name.clone()))
        .unwrap_or(profile_name.cloned());

    let manifest_profile = profile_name
        .as_ref()
        .and_then(|profile_name| manifest_profiles.get(profile_name));

    let (profile, manifest_profile) = match (global_profile, manifest_profile) {
        (Some(mut profile), Some(manifest_profile)) => {
            let profile_name = &profile.name;
            let manifest_is_cloud = profile_name.is_builtin_cloud() || manifest_profile.is_cloud();
            match &mut profile.profile {
                Profile::Golem(ref mut profile) => {
                    if manifest_is_cloud {
                        bail!("Profile {} is a global OSS profile, cannot be used as Cloud profile in the manifest.", profile_name);
                    }
                    if let Some(format) = &manifest_profile.format {
                        profile.config.default_format = *format;
                    }

                    // TODO: should we allow these? or show only warn logs?
                    if manifest_profile.url.is_some() {
                        bail!(
                            "Cannot override {} for global profile {} in application manifest!",
                            "url".log_color_highlight(),
                            profile_name.0.log_color_highlight()
                        );
                    }
                    if manifest_profile.worker_url.is_some() {
                        bail!(
                            "Cannot override {} for global profile {} in application manifest!",
                            "worker url".log_color_highlight(),
                            profile_name.0.log_color_highlight()
                        );
                    }
                }
                Profile::GolemCloud(ref mut profile) => {
                    if !manifest_is_cloud {
                        bail!("Profile {} is a global Cloud profile, cannot be used as OSS profile in the manifest.", profile_name);
                    }
                    if let Some(format) = &manifest_profile.format {
                        profile.config.default_format = *format;
                    }
                    // NOTE: no need to check urls, those are validated and forbidden for cloud
                }
            }
            (profile, Some(manifest_profile.clone()))
        }
        (Some(profile), None) => (profile, None),
        (None, Some(manifest_profile)) => {
            // If we only found manifest profile, then it must be found by name
            let profile_name = profile_name.unwrap();
            let manifest_is_cloud = profile_name.is_builtin_cloud() || manifest_profile.is_cloud();

            let profile_config = {
                let mut config = ProfileConfig::default();
                if let Some(format) = &manifest_profile.format {
                    config.default_format = *format
                }
                config
            };

            if manifest_is_cloud {
                let base_cloud_profile_with_name =
                    Config::get_profile(config_dir, &cloud_profile_name)?
                        .expect("Missing default cloud profile");

                let Profile::GolemCloud(base_cloud_profile) = base_cloud_profile_with_name.profile
                else {
                    unreachable!("Default cloud profile has wrong kind")
                };

                let profile = CloudProfile {
                    config: profile_config,
                    ..base_cloud_profile
                };
                (
                    NamedProfile {
                        name: profile_name,
                        profile: Profile::GolemCloud(profile),
                    },
                    Some(manifest_profile.clone()),
                )
            } else {
                let profile = {
                    let mut profile = OssProfile::default();
                    if let Some(url) = &manifest_profile.url {
                        profile.url = url.clone();
                    }
                    if let Some(worker_url) = &manifest_profile.worker_url {
                        profile.worker_url = Some(worker_url.clone());
                    }
                    profile
                };
                (
                    NamedProfile {
                        name: profile_name,
                        profile: Profile::Golem(profile),
                    },
                    Some(manifest_profile.clone()),
                )
            }
        }
        (None, None) => {
            // If no profile is found, then its name must be defined, as otherwise the global
            // default should have returned
            let profile_name = profile_name.as_ref().unwrap().clone();
            bail!(ContextInitHintError::ProfileNotFound {
                profile_name,
                manifest_profile_names: manifest_profiles.keys().cloned().collect(),
            });
        }
    };

    Ok((available_profile_names, profile, manifest_profile))
}

#[cfg(test)]
mod test {
    use crate::context::Context;
    use std::marker::PhantomData;
    use test_r::test;

    struct CheckSend<T: Send>(PhantomData<T>);
    struct CheckSync<T: Sync>(PhantomData<T>);

    #[test]
    fn test_context_is_send_sync() {
        let _ = CheckSend::<Context>(PhantomData);
        let _ = CheckSync::<Context>(PhantomData);
    }
}
