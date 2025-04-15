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

use crate::migration::IncludedMigrationsDir;
use crate::router::start_router;
use crate::StartedComponents;
use anyhow::Context;
use golem_common::config::DbConfig;
use golem_common::config::DbSqliteConfig;
use golem_component_compilation_service::config::DynamicComponentServiceConfig;
use golem_component_service::config::ComponentServiceConfig;
use golem_component_service::ComponentService;
use golem_component_service_base::config::ComponentCompilationEnabledConfig;
use golem_service_base::config::BlobStorageConfig;
use golem_service_base::config::LocalFileSystemBlobStorageConfig;
use golem_service_base::service::routing_table::RoutingTableConfig;
use golem_shard_manager::shard_manager_config::{
    FileSystemPersistenceConfig, HealthCheckConfig, PersistenceConfig, ShardManagerConfig,
};
use golem_worker_executor_base::services::additional_config::{
    ComponentServiceGrpcConfig, DefaultAdditionalGolemConfig,
};
use golem_worker_executor_base::services::golem_config::{
    CompiledComponentServiceConfig, IndexedStorageKVStoreSqliteConfig,
};
use golem_worker_executor_base::services::golem_config::{
    CompiledComponentServiceEnabledConfig, ShardManagerServiceConfig,
};
use golem_worker_executor_base::services::golem_config::{
    GolemConfig, IndexedStorageConfig, KeyValueStorageConfig,
};
use golem_worker_executor_base::services::golem_config::{
    PluginServiceConfig, PluginServiceGrpcConfig, ShardManagerServiceGrpcConfig,
};
use golem_worker_service::WorkerService;
use golem_worker_service_base::app_config::WorkerServiceBaseConfig;
use opentelemetry::global;
use opentelemetry_sdk::metrics::MeterProviderBuilder;
use prometheus::Registry;
use std::path::PathBuf;
use tokio::runtime::Handle;
use tokio::task::JoinSet;
use tracing::Instrument;

pub struct LaunchArgs {
    pub router_addr: String,
    pub router_port: u16,
    pub custom_request_port: u16,
    pub data_dir: PathBuf,
}

pub async fn launch_golem_services(args: &LaunchArgs) -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install crypto provider");

    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(Registry::default())
        .build()?;

    global::set_meter_provider(
        MeterProviderBuilder::default()
            .with_reader(exporter)
            .build(),
    );

    let mut join_set: JoinSet<anyhow::Result<()>> = JoinSet::new();

    tokio::fs::create_dir_all(&args.data_dir)
        .await
        .with_context(|| {
            format!(
                "Failed to create data directory at {}",
                args.data_dir.display()
            )
        })?;

    let started_components = start_components(args, &mut join_set).await?;

    start_router(
        &args.router_addr,
        args.router_port,
        started_components,
        &mut join_set,
    )?;

    while let Some(res) = join_set.join_next().await {
        res??;
    }

    Ok(())
}

async fn start_components(
    args: &LaunchArgs,
    join_set: &mut JoinSet<anyhow::Result<()>>,
) -> Result<StartedComponents, anyhow::Error> {
    let shard_manager = run_shard_manager(shard_manager_config(args), join_set).await?;

    let component_compilation_service =
        run_component_compilation_service(component_compilation_service_config(args), join_set)
            .await?;
    let component_service = run_component_service(
        component_service_config(args, &component_compilation_service),
        join_set,
    )
    .await?;
    let worker_executor = {
        let (config, additional_config) =
            worker_executor_config(args, &shard_manager, &component_service);
        run_worker_executor(config, additional_config, join_set).await?
    };
    let worker_service = run_worker_service(
        worker_service_config(args, &shard_manager, &component_service),
        join_set,
    )
    .await?;

    Ok(StartedComponents {
        shard_manager,
        worker_executor,
        component_service,
        worker_service,
        prometheus_registry: prometheus::default_registry().clone(),
    })
}

fn blob_storage_config(args: &LaunchArgs) -> BlobStorageConfig {
    BlobStorageConfig::LocalFileSystem(LocalFileSystemBlobStorageConfig {
        root: args.data_dir.join("blobs"),
    })
}

fn shard_manager_config(args: &LaunchArgs) -> ShardManagerConfig {
    ShardManagerConfig {
        grpc_port: 0,
        http_port: 0,
        persistence: PersistenceConfig::FileSystem(FileSystemPersistenceConfig {
            path: args.data_dir.join("sharding.bin"),
        }),
        health_check: HealthCheckConfig {
            silent: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn component_compilation_service_config(
    args: &LaunchArgs,
) -> golem_component_compilation_service::config::ServerConfig {
    golem_component_compilation_service::config::ServerConfig {
        component_service:
            golem_component_compilation_service::config::ComponentServiceConfig::Dynamic(
                DynamicComponentServiceConfig::default(),
            ),
        compiled_component_service: CompiledComponentServiceConfig::Enabled(
            CompiledComponentServiceEnabledConfig {},
        ),
        blob_storage: blob_storage_config(args),
        grpc_port: 0,
        http_port: 0,
        ..Default::default()
    }
}

fn component_service_config(
    args: &LaunchArgs,
    component_compilation_service: &golem_component_compilation_service::RunDetails,
) -> ComponentServiceConfig {
    ComponentServiceConfig {
        http_port: 0,
        grpc_port: 0,
        db: DbConfig::Sqlite(DbSqliteConfig {
            database: args
                .data_dir
                .join("components.db")
                .to_string_lossy()
                .to_string(),
            max_connections: 4,
        }),
        blob_storage: blob_storage_config(args),
        compilation: golem_component_service_base::config::ComponentCompilationConfig::Enabled(
            ComponentCompilationEnabledConfig {
                host: args.router_addr.clone(),
                port: component_compilation_service.grpc_port,
                retries: Default::default(),
                connect_timeout: Default::default(),
            },
        ),
        ..Default::default()
    }
}

fn worker_executor_config(
    args: &LaunchArgs,
    shard_manager_run_details: &golem_shard_manager::RunDetails,
    component_service_run_details: &golem_component_service::TrafficReadyEndpoints,
) -> (GolemConfig, DefaultAdditionalGolemConfig) {
    let mut config = GolemConfig {
        port: 0,
        http_port: 0,
        key_value_storage: KeyValueStorageConfig::Sqlite(DbSqliteConfig {
            database: args
                .data_dir
                .join("kv-store.db")
                .to_string_lossy()
                .to_string(),
            max_connections: 4,
        }),
        indexed_storage: IndexedStorageConfig::KVStoreSqlite(IndexedStorageKVStoreSqliteConfig {}),
        blob_storage: blob_storage_config(args),
        compiled_component_service: CompiledComponentServiceConfig::Enabled(
            CompiledComponentServiceEnabledConfig {},
        ),
        shard_manager_service: ShardManagerServiceConfig::Grpc(ShardManagerServiceGrpcConfig {
            host: args.router_addr.clone(),
            port: shard_manager_run_details.grpc_port,
            ..ShardManagerServiceGrpcConfig::default()
        }),
        plugin_service: PluginServiceConfig::Grpc(PluginServiceGrpcConfig {
            host: args.router_addr.clone(),
            port: component_service_run_details.grpc_port,
            ..Default::default()
        }),
        ..Default::default()
    };

    config.add_port_to_tracing_file_name_if_enabled();

    let additional_config = DefaultAdditionalGolemConfig {
        component_service:
            golem_worker_executor_base::services::additional_config::ComponentServiceConfig::Grpc(
                ComponentServiceGrpcConfig {
                    host: args.router_addr.clone(),
                    port: component_service_run_details.grpc_port,
                    ..ComponentServiceGrpcConfig::default()
                },
            ),
        ..Default::default()
    };

    (config, additional_config)
}

fn worker_service_config(
    args: &LaunchArgs,
    shard_manager_run_details: &golem_shard_manager::RunDetails,
    component_service_run_details: &golem_component_service::TrafficReadyEndpoints,
) -> WorkerServiceBaseConfig {
    WorkerServiceBaseConfig {
        port: 0,
        worker_grpc_port: 0,
        custom_request_port: args.custom_request_port,
        db: DbConfig::Sqlite(DbSqliteConfig {
            database: args
                .data_dir
                .join("workers.db")
                .to_string_lossy()
                .to_string(),
            max_connections: 4,
        }),
        gateway_session_storage:
            golem_worker_service_base::app_config::GatewaySessionStorageConfig::Sqlite(
                DbSqliteConfig {
                    database: args
                        .data_dir
                        .join("gateway-sessions.db")
                        .to_string_lossy()
                        .to_string(),
                    max_connections: 4,
                },
            ),
        blob_storage: blob_storage_config(args),
        component_service: golem_worker_service_base::app_config::ComponentServiceConfig {
            host: args.router_addr.clone(),
            port: component_service_run_details.grpc_port,
            ..golem_worker_service_base::app_config::ComponentServiceConfig::default()
        },
        routing_table: RoutingTableConfig {
            host: args.router_addr.clone(),
            port: shard_manager_run_details.grpc_port,
            ..RoutingTableConfig::default()
        },
        ..Default::default()
    }
}

async fn run_shard_manager(
    config: ShardManagerConfig,
    join_set: &mut JoinSet<anyhow::Result<()>>,
) -> Result<golem_shard_manager::RunDetails, anyhow::Error> {
    let prometheus_registry = prometheus::default_registry().clone();
    let span = tracing::info_span!("shard-manager");
    golem_shard_manager::run(&config, prometheus_registry, join_set)
        .instrument(span)
        .await
}

async fn run_component_compilation_service(
    config: golem_component_compilation_service::config::ServerConfig,
    join_set: &mut JoinSet<anyhow::Result<()>>,
) -> Result<golem_component_compilation_service::RunDetails, anyhow::Error> {
    let prometheus_registry = golem_component_compilation_service::metrics::register_all();
    let span = tracing::info_span!("component-compilation-service");
    golem_component_compilation_service::run(config, prometheus_registry, join_set)
        .instrument(span)
        .await
}

async fn run_component_service(
    config: ComponentServiceConfig,
    join_set: &mut JoinSet<anyhow::Result<()>>,
) -> Result<golem_component_service::TrafficReadyEndpoints, anyhow::Error> {
    let prometheus_registry = golem_component_service::metrics::register_all();
    let migrations_dir = IncludedMigrationsDir::new(ComponentService::db_migrations());
    let span = tracing::info_span!("component-service", component = "component-service");
    ComponentService::new(config, prometheus_registry, migrations_dir)
        .instrument(span.clone())
        .await?
        .start_endpoints(join_set)
        .instrument(span)
        .await
}

async fn run_worker_executor(
    config: GolemConfig,
    additional_config: DefaultAdditionalGolemConfig,
    join_set: &mut JoinSet<anyhow::Result<()>>,
) -> Result<golem_worker_executor_base::RunDetails, anyhow::Error> {
    let prometheus_registry = golem_worker_executor_base::metrics::register_all();

    let span = tracing::info_span!("worker-executor");
    golem_worker_executor::run(
        config,
        additional_config,
        prometheus_registry,
        Handle::current(),
        join_set,
    )
    .instrument(span)
    .await
}

async fn run_worker_service(
    config: WorkerServiceBaseConfig,
    join_set: &mut JoinSet<anyhow::Result<()>>,
) -> Result<golem_worker_service::TrafficReadyEndpoints, anyhow::Error> {
    let prometheus_registry = golem_worker_executor_base::metrics::register_all();
    let migration_path = IncludedMigrationsDir::new(WorkerService::db_migrations());
    let span = tracing::info_span!("worker-service");
    WorkerService::new(config, prometheus_registry, migration_path)
        .instrument(span.clone())
        .await?
        .start_endpoints(join_set)
        .instrument(span)
        .await
}
