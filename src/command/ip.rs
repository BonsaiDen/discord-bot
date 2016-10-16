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

        let response = match resolve_ip() {
            Ok(ip) => format!(
                "{} has requested my public IP address which is: {}.",
                member.nickname,
                ip
            ),
            Err(_) => "{} has requested my public IP address, but the lookup failed.".to_string()
        };

        self.delete_and_send(command.message, MessageActions::Send::public(
            &command.message,
            response
        ))

    }

}


// Helpers --------------------------------------------------------------------
fn resolve_ip() -> Result<String, String> {

    let client = Client::new();
    client.get("https://icanhazip.com/")
        .header(Connection::close())
        .send()
        .map_err(|err| err.to_string())
        .and_then(|mut res| {
            let mut body = String::new();
            res.read_to_string(&mut body)
               .map_err(|err| err.to_string())
               .map(|_| body)

        }).and_then(|body| {
            Ok(body.trim().to_string())
        })

}

