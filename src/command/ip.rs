// STD Dependencies -----------------------------------------------------------
use std::io::Read;


// External Dependencies ------------------------------------------------------
use hyper::Client;
use hyper::header::Connection;


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::{Member, Server};
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, MessageActions};


// Command Implementation -----------------------------------------------------
pub struct CommandImpl;

impl CommandHandler for CommandImpl {

    fn run(
        &self,
        command: Command,
        _: &Server,
        member: &Member,
        _: &BotConfig

    ) -> ActionGroup {
        self.delete_and_send(command.message, MessageActions::Send::public(
            &command.message,
            format!(
                "{} has requested my public IP address which is: {}.",
                member.nickname,
                get_ip().trim()
            )
        ))
    }

}

// Helpers --------------------------------------------------------------------
fn get_ip() -> String {

    let client = Client::new();
    let mut res = client.get(
        "https://icanhazip.com/"

    ).header(Connection::close()).send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    body

}

