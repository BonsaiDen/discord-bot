// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, MessageActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {

        if command.arguments.is_empty() {

            let mut lines: Vec<(&str, String)> = command.all_commands.iter().map(|(name, handler)| {
                (*name, format!("- **`{}`**: {}", name, handler.help()))

            }).collect();

            lines.sort();

            lines.insert(0, ("", "__Available Commands__\n".to_string()));
            lines.push(("", "\nType `!help <command_name>` for additional usage information.".to_string()));

            MessageActions::Send::private(
                &command.message,
                lines.into_iter().map(|(_, m)| m).collect::<Vec<String>>().join("\n")
            )

        } else if let Some(handler) = command.all_commands.get(command.arguments[0].as_str()) {
            handler.usage(command)

        } else {
            MessageActions::Send::private(
                &command.message,
                format!(
                    "**`{}`** is not a known command. Type `!help` for a listing of all available commands.",
                    command.arguments[0]
                )
            )
        }

    }

    fn help(&self) -> &str {
        "Show general help or usage information for a specific command."
    }

    fn usage(&self, command: Command) -> ActionGroup {
        MessageActions::Send::private(
            &command.message,
            "Usage: `!help [<command_name>]`".to_string()
        )
    }

}

