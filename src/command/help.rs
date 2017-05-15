// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, MessageActions};


// Statics --------------------------------------------------------------------
static HELP_TEXT_ONE: &'static str = "
**The following commands are currently available:**

- `!s <sound>` - Plays the requested sound immediately. See below for details on what `<sound>` can be.
- `!q <sound>` - Queues up the requested sound to be played once all other currently playing / queued sounds have finished.
- `!sounds` [<pattern>, ...] - Show a list of all available sound effects (matching the specified patterns) in a private channel.
- `!delete <effect>` - Deletes the specified sound effect.
- `!rename <old_effect_name> <new_effect_name>` - Renames the specified sound effect.
- `!silence` - Immediately stops all playing sounds and removes all other queued effects.
- `!greeting <add|remove> <user#ident> [<effect_name>]` - Adds or remove a custom greeting for a user.
- `!greetings` - Show a list of all existing custom user greetings in a private channel.
- `!uploader <add|remove> <user#ident> [<effect_name>]` - Adds or remove a user from the list of uploaders for the current server.
- `!uploaders` - Show a list of all users which are whitelisted to upload sound effects to the current server.
- `!alias <add|remove> <name> [<effect_name>, ...]` - Adds or remove a effect alias.
- `!aliases` - Show a list of all existing effect aliases in a private channel.
- `!bans` - Show a list of all banned users for the current server.
- `!ban <add|remove> <user#ident>` - Add or remove user bans.
- `!leave` - Makes the bot leave its current voice channel.
- `!pin` - Pins the bot to its current voice channel.
- `!ip` - Posts the bot's the public IP onto the current channel.
- `!reload` - Reloads the sound list from the on disk flac files.
- `!help` - Displays this help text.
";

static HELP_TEXT_TWO: &'static str = "**Sound Effects**

Sound effects can be played by requesting them via the `!s <sound>` command,
where `<sound>` can either be the *full name*, a *group prefix*, or a *wildcard*.

- `full name` - Simply check the `!sounds` listing.
- `group prefix` - This is the part of a sound name before the first `_`  character (e.g. `siw` is the *group prefix* for `siw_kaffee` etc.), requesting a *group prefix* will select a random sound from the group.
- `wildcards` - These are either `*` for any random sound or `*word*` for any sound effect which contains the specified word part.

**Effect File Uploads**

Sound effects can be directly uploaded by whitelisted users via the default discord file upload feature.

The filename must be at least 3 characters long, the file extension must be `flac` and the file itself must be a valid flac file encoded at 48khz and 16bit with at most 2 MiB.

Also, a effect with the same name may not yet exist.";


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {
        vec![
            MessageActions::Send::single_private(
                &command.message, HELP_TEXT_ONE.to_string()
            ),
            MessageActions::Send::single_private(
                &command.message, HELP_TEXT_TWO.to_string()
            )
        ]
    }

}

