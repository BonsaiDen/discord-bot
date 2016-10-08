// STD Dependencies -----------------------------------------------------------
use std::io::Read;


// External Dependencies ------------------------------------------------------
use hyper::Client;
use hyper::header::Connection;


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::member::Member;
use ::core::server::Server;
use ::command::{Command, CommandImplementation};
use ::actions::{ActionGroup, DeleteMessage, SendPublicMessage};


// Command Implementation -----------------------------------------------------
pub struct IpCommand;

impl CommandImplementation for IpCommand {

    fn run(
        &self,
        command: Command,
        _: &Server,
        member: &Member,
        _: &BotConfig

    ) -> ActionGroup {
        vec![
            DeleteMessage::new(command.message),
            SendPublicMessage::new(
                &command.message,
                format!(
                    "{} has requested my public IP address which is: {}.",
                    member.nickname,
                    get_ip().trim()
                )
            )
        ]
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

