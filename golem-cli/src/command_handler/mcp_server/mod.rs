use std::{sync::Arc, time::Duration};

use crate::{
    command::mcp_server::{McpServerSubcommand, Transport},
    context::Context,
};

use rmcp::{
    handler::server::tool::ToolRouter,
    transport::{SseServer, StreamableHttpServerConfig},
};

use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
    service::TowerToHyperService,
};
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpService,
};

mod handler;
mod tools;

pub struct McpServerCommandHandler {
    _ctx: Arc<Context>,
}

impl McpServerCommandHandler {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { _ctx: ctx }
    }

    pub async fn handle_command(&self, subcommand: McpServerSubcommand) -> anyhow::Result<()> {
        match subcommand {
            McpServerSubcommand::Run {
                port,
                timeout,
                transport,
            } => self.mcp_server_start(port, timeout, transport).await,
        }
    }

    async fn mcp_server_start(
        &self,
        port: Option<u16>,
        timeout: Option<u64>,
        transport: Option<Transport>,
    ) -> anyhow::Result<()> {
        let port = port.unwrap_or(8080);
        let timeout = timeout.unwrap_or(6);

        let transport_to_use = transport.unwrap_or(Transport::Sse);

        match transport_to_use {
            Transport::StreamableHttp => {
                let service = TowerToHyperService::new(StreamableHttpService::new(
                    || Ok(GolemCliMcpServer::new()),
                    LocalSessionManager::default().into(),
                    StreamableHttpServerConfig {
                        sse_keep_alive: Some(Duration::new(timeout, 0)),
                        ..Default::default()
                    },
                ));
                let listener = tokio::net::TcpListener::bind(format!("[::1]:{}", port)).await?;
                loop {
                    let io = tokio::select! {
                        _ = tokio::signal::ctrl_c() => break,
                        accept = listener.accept() => {
                            TokioIo::new(accept?.0)
                        }
                    };
                    let service = service.clone();
                    tokio::spawn(async move {
                        let _result = Builder::new(TokioExecutor::default())
                            .serve_connection(io, service)
                            .await;
                    });
                }
            }
            Transport::Sse => {
                let ct = SseServer::serve(format!("127.0.0.1:{}", port).parse()?)
                    .await?
                    .with_service_directly(GolemCliMcpServer::new);

                tokio::signal::ctrl_c().await?;
                ct.cancel();
            }
        }
        Ok(())
    }
}

pub struct GolemCliMcpServer {
    tool_router: ToolRouter<GolemCliMcpServer>,
}

impl GolemCliMcpServer {
    fn new() -> Self {
        Self {
            tool_router: self::GolemCliMcpServer::tool_router_app()
                + self::GolemCliMcpServer::tool_router_component()
                + self::GolemCliMcpServer::tool_router_component_plugin()
                + self::GolemCliMcpServer::tool_router_api()
                + self::GolemCliMcpServer::tool_router_api_definition()
                + self::GolemCliMcpServer::tool_router_api_deployment()
                // + self::GolemCliMcpServer::tool_router_api_security_scheme()
                // + self::GolemCliMcpServer::tool_router_api_cloud_certificate()
                + self::GolemCliMcpServer::tool_router_api_cloud_domain()
                + self::GolemCliMcpServer::tool_router_cloud_account()
                + self::GolemCliMcpServer::tool_router_cloud_account_grant()
                + self::GolemCliMcpServer::tool_router_cloud_project_plugin()
                + self::GolemCliMcpServer::tool_router_cloud_project_policy()
                // + self::GolemCliMcpServer::tool_router_cloud_token()
                + self::GolemCliMcpServer::tool_router_plugin()
                + self::GolemCliMcpServer::tool_router_profile()
                + self::GolemCliMcpServer::tool_router_profile_config()
                + self::GolemCliMcpServer::tool_router_worker()
                + self::GolemCliMcpServer::tool_router_repl(),
        }
    }
}
