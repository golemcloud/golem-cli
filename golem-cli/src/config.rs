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

use crate::cloud::CloudAuthenticationConfig;
use crate::model::{Format, GolemError, HasFormatConfig, ProfileType};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::warn;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub profiles: HashMap<ProfileName, Profile>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub active_profile: Option<ProfileName>,
    // TODO: deprecate this?
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub active_cloud_profile: Option<ProfileName>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct ProfileName(pub String);

impl Display for ProfileName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ProfileName {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl From<String> for ProfileName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedProfile {
    pub name: ProfileName,
    pub profile: Profile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Profile {
    Golem(OssProfile),
    GolemCloud(CloudProfile),
}

impl Profile {
    pub fn config(self) -> ProfileConfig {
        match self {
            Profile::Golem(p) => p.config,
            Profile::GolemCloud(p) => p.config,
        }
    }

    pub fn get_config(&self) -> &ProfileConfig {
        match self {
            Profile::Golem(p) => &p.config,
            Profile::GolemCloud(p) => &p.config,
        }
    }

    pub fn get_config_mut(&mut self) -> &mut ProfileConfig {
        match self {
            Profile::Golem(p) => &mut p.config,
            Profile::GolemCloud(p) => &mut p.config,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CloudProfile {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub custom_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub custom_cloud_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub custom_worker_url: Option<Url>,
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub allow_insecure: bool,
    #[serde(default)]
    pub config: ProfileConfig,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub auth: Option<CloudAuthenticationConfig>,
}

impl HasFormatConfig for CloudProfile {
    fn format(&self) -> Option<Format> {
        Some(self.config.default_format)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OssProfile {
    pub url: Url,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub worker_url: Option<Url>,
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub allow_insecure: bool,
    #[serde(default)]
    pub config: ProfileConfig,
}

impl HasFormatConfig for OssProfile {
    fn format(&self) -> Option<Format> {
        Some(self.config.default_format)
    }
}

// TODO: flatten this?
#[derive(Debug, Clone, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct ProfileConfig {
    #[serde(default)]
    pub default_format: Format,
}

impl Config {
    fn config_path(config_dir: &Path) -> PathBuf {
        config_dir.join("config.json")
    }

    fn read_from_file_opt(config_dir: &Path) -> Option<Config> {
        let file = File::open(Self::config_path(config_dir)).ok()?;
        let reader = BufReader::new(file);

        let parsed: serde_json::Result<Config> = serde_json::from_reader(reader);

        match parsed {
            Ok(conf) => Some(conf),
            Err(err) => {
                warn!("Config parsing failed: {err}");
                // TODO: should not silently ignore config parsing errors
                None
            }
        }
    }

    pub fn read_from_file(config_dir: &Path) -> Config {
        Self::read_from_file_opt(config_dir).unwrap_or_default()
    }

    fn store_file(&self, config_dir: &Path) -> Result<(), GolemError> {
        create_dir_all(config_dir)
            .map_err(|err| GolemError(format!("Can't create config directory: {err}")))?;

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(true)
            .open(Self::config_path(config_dir))
            .map_err(|err| GolemError(format!("Can't open config file: {err}")))?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self)
            .map_err(|err| GolemError(format!("Can't save config to file: {err}")))
    }

    pub fn set_active_profile_name(
        profile_name: ProfileName,
        config_dir: &Path,
    ) -> Result<(), GolemError> {
        let mut config = Self::read_from_file(config_dir);

        let Some(profile) = config.profiles.get(&profile_name) else {
            return Err(GolemError(format!(
                "No profile {profile_name} in configuration. Available profiles: [{}]",
                config.profiles.keys().map(|n| &n.0).join(", ")
            )));
        };

        match &profile {
            Profile::Golem(_) => config.active_profile = Some(profile_name),
            Profile::GolemCloud(_) => config.active_cloud_profile = Some(profile_name),
        }

        config.store_file(config_dir)?;

        Ok(())
    }

    pub fn get_active_profile(
        config_dir: &Path,
        selected_profile: Option<ProfileName>,
    ) -> NamedProfile {
        // TODO: allow missing config
        let mut config = Self::read_from_file(config_dir);

        let name = selected_profile.unwrap_or_else(|| {
            config
                .active_profile
                .unwrap_or_else(|| panic!("TODO: handle builtin (local and cloud) profiles"))
        });

        NamedProfile {
            name: name.clone(),
            profile: config.profiles.remove(&name).unwrap(),
        }
    }

    pub fn get_profile(name: &ProfileName, config_dir: &Path) -> Option<Profile> {
        let mut config = Self::read_from_file(config_dir);
        config.profiles.remove(name)
    }

    pub fn set_profile(
        name: ProfileName,
        profile: Profile,
        config_dir: &Path,
    ) -> Result<(), GolemError> {
        let mut config = Self::read_from_file(config_dir);

        let _ = config.profiles.insert(name, profile);

        config.store_file(config_dir)
    }

    pub fn delete_profile(name: &ProfileName, config_dir: &Path) -> Result<(), GolemError> {
        /*let mut config = Self::read_from_file(config_dir);

        if &config
            .active_profile
            .clone()
            .unwrap_or_else(|| ProfileName::default(CliKind::Universal))
            == name
        {
            return Err(GolemError("Can't remove active profile".to_string()));
        }

        if &config
            .active_cloud_profile
            .clone()
            .unwrap_or_else(|| ProfileName::default(CliKind::Cloud))
            == name
        {
            return Err(GolemError("Can't remove active cloud profile".to_string()));
        }

        let _ = config
            .profiles
            .remove(name)
            .ok_or(GolemError(format!("Profile {name} not found")))?;

        config.store_file(config_dir)
        */
        todo!()
    }
}

pub struct ClientConfig {
    pub component_url: Url,
    pub worker_url: Url,
    pub cloud_url: Option<Url>,
    pub service_http_client_config: HttpClientConfig,
    pub health_check_http_client_config: HttpClientConfig,
    pub file_download_http_client_config: HttpClientConfig,
}

impl From<&Profile> for ClientConfig {
    fn from(profile: &Profile) -> Self {
        match profile {
            Profile::Golem(profile) => {
                let allow_insecure = profile.allow_insecure;

                ClientConfig {
                    component_url: profile.url.clone(),
                    worker_url: profile
                        .worker_url
                        .clone()
                        .unwrap_or_else(|| profile.url.clone()),
                    cloud_url: None,
                    service_http_client_config: HttpClientConfig::new_for_service_calls(
                        allow_insecure,
                    ),
                    health_check_http_client_config: HttpClientConfig::new_for_health_check(
                        allow_insecure,
                    ),
                    file_download_http_client_config: HttpClientConfig::new_for_file_download(
                        allow_insecure,
                    ),
                }
            }
            Profile::GolemCloud(_profile) => {
                todo!()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub allow_insecure: bool,
    pub timeout: Option<Duration>,
    pub connect_timeout: Option<Duration>,
    pub read_timeout: Option<Duration>,
}

impl HttpClientConfig {
    pub fn new_for_service_calls(allow_insecure: bool) -> Self {
        Self {
            allow_insecure,
            timeout: None,
            connect_timeout: None,
            read_timeout: None,
        }
        .with_env_overrides("GOLEM_HTTP")
    }

    pub fn new_for_health_check(allow_insecure: bool) -> Self {
        Self {
            allow_insecure,
            timeout: Some(Duration::from_secs(2)),
            connect_timeout: Some(Duration::from_secs(1)),
            read_timeout: Some(Duration::from_secs(1)),
        }
        .with_env_overrides("GOLEM_HTTP_HEALTHCHECK")
    }

    pub fn new_for_file_download(allow_insecure: bool) -> Self {
        Self {
            allow_insecure,
            timeout: Some(Duration::from_secs(60)),
            connect_timeout: Some(Duration::from_secs(10)),
            read_timeout: Some(Duration::from_secs(60)),
        }
        .with_env_overrides("GOLEM_HTTP_FILE_DOWNLOAD")
    }

    fn with_env_overrides(mut self, prefix: &str) -> Self {
        fn env_duration(name: &str) -> Option<Duration> {
            let duration_str = std::env::var(name).ok()?;
            Some(iso8601::duration(&duration_str).ok()?.into())
        }

        let duration_fields: Vec<(&str, &mut Option<Duration>)> = vec![
            ("TIMEOUT", &mut self.timeout),
            ("CONNECT_TIMEOUT", &mut self.connect_timeout),
            ("READ_TIMEOUT", &mut self.read_timeout),
        ];

        for (env_var_name, field) in duration_fields {
            if let Some(duration) = env_duration(&format!("{}_{}", prefix, env_var_name)) {
                *field = Some(duration);
            }
        }

        self
    }
}
