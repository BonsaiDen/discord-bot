// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Modules --------------------------------------------------------------------
mod ip;
mod help;
mod alias;
mod leave;
mod reload;
mod sounds;
mod aliases;
mod silence;
mod greeting;
mod greetings;
mod not_found;
mod play_effect;
mod delete_effect;
mod rename_effect;


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::core::message::Message;
use ::actions::{ActionGroup, SendPrivateMessage};


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

        let command: Box<CommandImplementation> = match self.name.as_str() {
            "s" => Box::new(play_effect::PlayEffectCommand::instant()),
            "q" => Box::new(play_effect::PlayEffectCommand::queued()),
            "delete" => Box::new(delete_effect::DeleteEffectCommand),
            "rename" => Box::new(rename_effect::RenameEffectCommand),
            "sounds" => Box::new(sounds::SoundsCommand),
            "silence" => Box::new(silence::SilenceCommand),
            "alias" => Box::new(alias::AliasCommand),
            "aliases" => Box::new(aliases::AliasesCommand),
            "greeting" => Box::new(greeting::GreetingCommand),
            "greetings" => Box::new(greetings::GreetingsCommand),
            "ip" => Box::new(ip::IpCommand),
            "leave" => Box::new(leave::LeaveCommand),
            "reload" => Box::new(reload::ReloadCommand),
            "help" => Box::new(help::HelpCommand),
            _ => Box::new(not_found::NotFoundCommand)
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
pub trait CommandImplementation {

    // TODO add default()

    fn run(
        &self,
        _: Command,
        _: &Server,
        _: &Member,
        _: &BotConfig

    ) -> ActionGroup;

    fn requires_unique_server(&self, command: Command) -> ActionGroup {
        vec![SendPrivateMessage::new(
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
        vec![SendPrivateMessage::new(
            &command.message,
            format!(
                "The command `{}` requires bot admin rights on the current server.",
                command.name
            )
        )]
    }

}

