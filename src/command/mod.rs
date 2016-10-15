// STD Dependencies -----------------------------------------------------------
use std::fmt;


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
mod play;
mod reload;
mod rename;
mod sounds;
mod silence;


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::{Member, Message, Server};
use ::action::{Action, ActionGroup, MessageActions};


// Command Abstraction --------------------------------------------------------
#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub arguments: Vec<String>,
    pub message: Message
}


// Public Interface -----------------------------------------------------------
impl Command {

    pub fn new(
        name: String,
        arguments: Vec<String>,
        message: Message,

    ) -> Command {
        Command {
            name: name,
            arguments: arguments,
            message: message
        }
    }

    pub fn parse(
        self,
        server: &Server,
        member: &Member,
        config: &BotConfig

    ) -> ActionGroup {

        if member.is_banned {
            return vec![];
        }

        let command: Box<CommandHandler> = match self.name.as_str() {
            "s" => Box::new(play::CommandImpl::instant()),
            "q" => Box::new(play::CommandImpl::queued()),
            "delete" => Box::new(delete::CommandImpl),
            "rename" => Box::new(rename::CommandImpl),
            "sounds" => Box::new(sounds::CommandImpl),
            "silence" => Box::new(silence::CommandImpl),
            "alias" => Box::new(alias::CommandImpl),
            "aliases" => Box::new(aliases::CommandImpl),
            "greeting" => Box::new(greeting::CommandImpl),
            "greetings" => Box::new(greetings::CommandImpl),
            "ban" => Box::new(ban::CommandImpl),
            "bans" => Box::new(bans::CommandImpl),
            "ip" => Box::new(ip::CommandImpl),
            "leave" => Box::new(leave::CommandImpl),
            "reload" => Box::new(reload::CommandImpl),
            "help" => Box::new(help::CommandImpl),
            _ => Box::new(not_found::CommandImpl)
        };

        command.run(self, server, member, config)

    }

}


// Traits  --------------------------------------------------------------------
impl fmt::Display for Command {
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

    fn run(
        &self,
        _: Command,
        _: &Server,
        _: &Member,
        _: &BotConfig

    ) -> ActionGroup;

    fn requires_unique_server(&self, command: Command) -> ActionGroup {
        vec![MessageActions::Send::private(
            &command.message,
            format!(
                "The command `{}` requires a unique server as its target.
                Since you are a member of at least two bot-enabled servers,
                the command cannot be invoked from a private channel.
                Please re-issue the command from a public channels of the target server.",
                command.name
            )
        )]
    }

    fn requires_admin(&self, command: Command) -> ActionGroup {
        vec![MessageActions::Send::private(
            &command.message,
            format!(
                "The command `{}` requires bot admin rights on the current server.",
                command.name
            )
        )]
    }

    fn delete_and_send(&self, message: Message, action: Box<Action>) -> ActionGroup {
        vec![
            MessageActions::Delete::new(message),
            action
        ]
    }

}

