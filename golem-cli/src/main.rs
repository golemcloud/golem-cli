use golem_cli::command_handler::CommandHandler;
use std::process::ExitCode;

fn main() -> ExitCode {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime for cli main")
        .block_on(CommandHandler::handle_args(std::env::args_os()))
}
