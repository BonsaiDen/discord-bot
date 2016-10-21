// STD Dependencies -----------------------------------------------------------
use std::fmt;


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
mod aliases;
mod ban;
mod bans;
mod delete;
mod greeting;
mod greetings;
mod help;
mod ip;
mod leave;
mod not_found;
mod pin;
mod play;
mod reload;
mod rename;
mod sounds;
mod silence;


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::{Member, Message, Server};
use ::action::{ActionGroup, MessageActions};


// Command Abstraction --------------------------------------------------------
pub struct Command<'a> {
    pub name: String,
    pub arguments: Vec<String>,
    pub message: Message,
    pub server: &'a Server,
    pub member: &'a Member,
    pub config: &'a BotConfig
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
            config: config
        }
    }

    fn run<T: CommandHandler>(self, handler: T) -> ActionGroup {

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

        } else if argc < handler.require_min_arguments() {
            actions.append(&mut handler.usage(self));

        } else if argc != handler.require_exact_arguments().unwrap_or(argc) {
            actions.append(&mut handler.usage(self));

        } else {
            actions.append(&mut handler.run(self));
        }

        actions

    }

    pub fn process(self) -> ActionGroup {

        if self.member.is_banned {
            return vec![];

        } else {
            match self.name.as_str() {
                "s" => self.run(play::Handler::instant()),
                "q" => self.run(play::Handler::queued()),
                "delete" => self.run(delete::Handler),
                "rename" => self.run(rename::Handler),
                "sounds" => self.run(sounds::Handler),
                "silence" => self.run(silence::Handler),
                "alias" => self.run(alias::Handler),
                "aliases" => self.run(aliases::Handler),
                "greeting" => self.run(greeting::Handler),
                "greetings" => self.run(greetings::Handler),
                "ban" => self.run(ban::Handler),
                "bans" => self.run(bans::Handler),
                "pin" => self.run(pin::Handler),
                "ip" => self.run(ip::Handler),
                "leave" => self.run(leave::Handler),
                "reload" => self.run(reload::Handler),
                "help" => self.run(help::Handler),
                _ => self.run(not_found::Handler)
            }
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
pub trait CommandHandler {

    fn run(&self, _: Command) -> ActionGroup;

    fn usage(&self, _: Command) -> ActionGroup {
        vec![]
    }

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

