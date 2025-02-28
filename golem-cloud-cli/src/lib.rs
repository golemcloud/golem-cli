use golem_common::golem_version;

pub mod cloud;

pub const VERSION: &str = golem_version!();

#[cfg(test)]
test_r::enable!();

#[cfg(test)]
mod tests {
    use test_r::test;

    use crate::cloud::cli::GolemCloudCli;
    use clap::{Command, CommandFactory};
    use golem_cli::command_old::profile::CloudProfileAdd;

    // TODO: delete before merge
    #[test]
    fn dump_commands() {
        let command = GolemCloudCli::<CloudProfileAdd>::command();
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
