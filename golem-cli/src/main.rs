use async_trait::async_trait;
use golem_cli::command::server::ServerSubcommand;
use golem_cli::command_handler::{CommandHandler, CommandHandlerHooks};
use golem_cli::context::Context;
use std::process::ExitCode;
use std::sync::Arc;

struct NoHooks {}

#[async_trait]
impl CommandHandlerHooks for NoHooks {
    #[cfg(feature = "server-commands")]
    async fn handler_server_commands(
        &self,
        _ctx: Arc<Context>,
        _subcommand: ServerSubcommand,
    ) -> anyhow::Result<()> {
        unimplemented!()
    }
}

fn main() -> ExitCode {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime for golem-cli main")
        .block_on(CommandHandler::handle_args(
            std::env::args_os(),
            Arc::new(NoHooks {}),
        ))
}
