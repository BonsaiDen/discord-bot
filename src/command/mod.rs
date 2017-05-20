// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::collections::HashMap;


// Macros ---------------------------------------------------------------------
#[macro_export]
macro_rules! require_unique_server {
    () => (
        fn require_unique_server(&self) -> bool {
            true
        }
    );
}

#[macro_export]
macro_rules! require_server_admin {
    () => (
        fn require_server_admin(&self) -> bool {
            true
        }
    );
}

#[macro_export]
macro_rules! require_min_arguments {
    ($count:expr) => (
        fn require_min_arguments(&self) -> usize {
            $count
        }
    );
}

#[macro_export]
macro_rules! require_exact_arguments {
    ($count:expr) => (
        fn require_exact_arguments(&self) -> Option<usize> {
            Some($count)
        }
    );
}

#[macro_export]
macro_rules! delete_command_message {
    () => (
        fn delete_command_message(&self) -> bool {
            true
        }
    );
}


// Modules --------------------------------------------------------------------
mod alias;
mod ban;
mod delete;
mod greeting;
mod help;
mod ip;
mod leave;
mod not_found;
mod pin;
mod play;
mod record;
mod rename;
mod sounds;
mod silence;
mod streamer;
mod uploader;


// Statics --------------------------------------------------------------------
lazy_static! {
    static ref COMMANDS: HashMap<&'static str, Box<CommandHandler>> = {
        let mut m: HashMap<&'static str, Box<CommandHandler>> = HashMap::new();
        m.insert("alias", Box::new(alias::Handler));
        m.insert("ban", Box::new(ban::Handler));
        m.insert("delete", Box::new(delete::Handler));
        m.insert("greeting", Box::new(greeting::Handler));
        m.insert("ip", Box::new(ip::Handler));
        m.insert("leave", Box::new(leave::Handler));
        m.insert("pin", Box::new(pin::Handler));
        m.insert("s", Box::new(play::Handler::instant()));
        m.insert("q", Box::new(play::Handler::queued()));
        m.insert("help", Box::new(help::Handler));
        m.insert("record", Box::new(record::Handler));
        m.insert("rename", Box::new(rename::Handler));
        m.insert("silence", Box::new(silence::Handler));
        m.insert("sounds", Box::new(sounds::Handler));
        m.insert("streamer", Box::new(streamer::Handler));
        m.insert("uploader", Box::new(uploader::Handler));
        m
    };
}


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::server::Server;
use ::core::{Member, Message};
use ::action::{ActionGroup, MessageActions};


// Command Abstraction --------------------------------------------------------
pub struct Command<'a> {
    pub name: String,
    pub arguments: Vec<String>,
    pub message: Message,
    pub server: &'a Server,
    pub member: &'a Member,
    pub config: &'a BotConfig,
    pub all_commands: &'a HashMap<&'static str, Box<CommandHandler>>
}


// Public Interface -----------------------------------------------------------
impl<'a> Command<'a> {

    pub fn from_parts(
        name: String,
        arguments: Vec<String>,
        message: Message,
        server: &'a Server,
        member: &'a Member,
        config: &'a BotConfig

    ) -> Command<'a> {
        Command {
            name: name,
            arguments: arguments,
            message: message,
            server: server,
            member: member,
            config: config,
            all_commands: &COMMANDS
        }
    }

    fn run(self, handler: &Box<CommandHandler>) -> ActionGroup {

        let argc = self.arguments.len();
        let mut actions: ActionGroup = vec![];

        if handler.delete_command_message() {
            actions.push(MessageActions::Delete::new(self.message));
        }

        if handler.require_unique_server() && !self.message.has_unique_server() {
            actions.push(MessageActions::Send::single_private(
                &self.message,
                format!(
                    "The command `{}` requires a unique server as its target.
                    Since you are a member of at least two bot-enabled servers,
                    the command cannot be invoked from a private channel.
                    Please re-issue the command from a public channels of the target server.",
                    self.name
                )
            ));

        } else if handler.require_server_admin() && !self.member.is_admin {
            actions.push(MessageActions::Send::single_private(
                &self.message,
                format!(
                    "The command `{}` requires bot admin rights on the current server.",
                    self.name
                )
            ));

        } else if argc < handler.require_min_arguments()
               || argc != handler.require_exact_arguments().unwrap_or(argc) {

            actions.append(&mut handler.usage(self));

        } else {
            actions.append(&mut handler.run(self));
        }

        actions

    }

    pub fn process(self) -> ActionGroup {

        if self.member.is_banned {
            vec![]

        } else if let Some(handler) = COMMANDS.get(self.name.as_str()) {
            self.run(handler)

        } else {
            let not_found: Box<CommandHandler> = Box::new(not_found::Handler);
            self.run(&not_found)
        }

    }

}


// Traits  --------------------------------------------------------------------
impl<'a> fmt::Display for Command<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, "[Command \"{}\" with arguments <{}> from user #{} on #{}]",
            self.name,
            self.arguments.join(", "),
            self.message.user_id,
            self.message.server_id
        )
    }
}

// Command Implementation Trait -----------------------------------------------
pub trait CommandHandler: Sync {

    fn run(&self, _: Command) -> ActionGroup;

    fn usage(&self, _: Command) -> ActionGroup;

    fn help(&self) -> &str;

    fn require_unique_server(&self) -> bool {
        false
    }

    fn require_server_admin(&self) -> bool {
        false
    }

    fn require_min_arguments(&self) -> usize {
        0
    }

    fn require_exact_arguments(&self) -> Option<usize> {
        None
    }

    fn delete_command_message(&self) -> bool {
        false
    }

}

