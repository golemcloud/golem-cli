use golem_cli::command_handler::{CommandHandler, CommandHandlerHooks};
use std::process::ExitCode;
use std::sync::Arc;

struct NoHooks {}

impl CommandHandlerHooks for NoHooks {
    #[cfg(feature = "server-commands")]
    fn handler_server_commands(
        &self,
        _ctx: Arc<golem_cli::context::Context>,
        _subcommand: golem_cli::command::server::ServerSubcommand,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> {
        async { unimplemented!() }
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
