// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, EffectActions, MessageActions};


// Statics --------------------------------------------------------------------
static USAGE_TEXT: &str = "Usage: `!sounds [<effect_pattern>, ...]`

Lists all available sound effects that match the specified pattern(s).

Each **`effect_pattern`** can be one of the following variants:

- `full_sound_name` - Only the exactly matching effect.
- `prefix` - A random effect which name starts with the specified prefix, followed by an underscore.
- `*wildcard` - A random effect which *ends* with the specified wildcard.
- `wildcard*` - A random effect which *starts* with the specified wildcard.
- `*wildcard*` - A random effect which *contains* the specified wildcard.";


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    require_unique_server!();
    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {
        if command.arguments.is_empty() {
            vec![EffectActions::List::all(command.message)]

        } else {
            vec![EffectActions::List::matching(command.message, command.arguments)]
        }
    }

    fn help(&self) -> &str {
        "List available sound effects matching the specified pattern(s)."
    }

    fn usage(&self, command: Command) -> ActionGroup {
        MessageActions::Send::private(&command.message, USAGE_TEXT.to_string())
    }

}

