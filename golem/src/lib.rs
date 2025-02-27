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

use prometheus::Registry;

pub mod command;
pub mod launch;
mod migration;
mod router;

#[cfg(test)]
test_r::enable!();

pub struct StartedComponents {
    pub component_service: golem_component_service::TrafficReadyEndpoints,
    pub shard_manager: golem_shard_manager::RunDetails,
    pub worker_executor: golem_worker_executor_base::RunDetails,
    pub worker_service: golem_worker_service::TrafficReadyEndpoints,
    pub prometheus_registy: Registry,
}

#[cfg(test)]
mod tests {
    use test_r::test;

    use crate::command::SingleExecutableCommand;
    use clap::{Command, CommandFactory};
    use golem_cli::command::profile::OssProfileAdd;
    use golem_cli::oss::cli::GolemOssCli;

    // TODO: delete before merge
    #[test]
    fn dump_commands() {
        let command = GolemOssCli::<OssProfileAdd, SingleExecutableCommand>::command();
        dump_command(0, &command);
    }

    fn dump_command(level: usize, command: &Command) {
        print!("{}{}", "\t".repeat(level), command.get_name());
        let (positional, flag_args): (Vec<_>, Vec<_>) =
            command.get_arguments().partition(|arg| arg.is_positional());

        if !positional.is_empty() {
            for arg in positional {
                print!(" <{}>", arg.get_id());
            }
        }
        println!();

        if !flag_args.is_empty() {
            print!("{}", "\t".repeat(level + 2));
            for arg in flag_args.clone() {
                print!(" --{}", arg.get_long().unwrap(),);
                arg.get_short()
                    .iter()
                    .for_each(|short| print!("({})", short));
            }
            println!()
        }

        let subcommand_level = level + 1;
        for subcommand in command.get_subcommands() {
            dump_command(subcommand_level, subcommand);
        }
    }
}
