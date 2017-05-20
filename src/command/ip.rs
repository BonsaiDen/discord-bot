// STD Dependencies -----------------------------------------------------------
use std::io::Read;


// External Dependencies ------------------------------------------------------
use hyper::Client;
use hyper::header::Connection;


// Internal Dependencies ------------------------------------------------------
use ::command::{Command, CommandHandler};
use ::action::{ActionGroup, MessageActions};


// Command Implementation -----------------------------------------------------
pub struct Handler;

impl CommandHandler for Handler {

    delete_command_message!();

    fn run(&self, command: Command) -> ActionGroup {

        let response = match resolve_ip() {
            Ok(ip) => format!(
                "{} has requested my public IP address which is: {}.",
                command.member.nickname,
                ip
            ),
            Err(_) => "{} has requested my public IP address, but the lookup failed.".to_string()
        };

        MessageActions::Send::public(
            &command.message,
            response
        )

    }

    fn help(&self) -> &str {
        "Post the bot's current IP address onto the channel."
    }

    fn usage(&self, command: Command) -> ActionGroup {
        MessageActions::Send::private(
            &command.message,
            "Usage: `!ip`".to_string()
        )
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

