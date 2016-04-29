// STD Dependencies -----------------------------------------------------------
use std::io::Read;


// External Dependencies ------------------------------------------------------
use hyper::Client;
use hyper::header::Connection;


// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// Bot IP Information ---------------------------------------------------------
pub struct Ip;


// Command Implementation -----------------------------------------------------
impl Command for Ip {

    fn execute(&self, _: &mut Handle, server: &mut Server, user: &User) -> CommandResult {
        info!("[{}] [{}] [Command] [IP] Performing IP lookup...", server, user);
        let ip = get_ip();
        Some(vec![
            format!("{} has requested my public IP address which is: {}.", user.nickname, ip.trim())
        ])
    }

    fn requires_unique_server(&self) -> bool {
        false
    }

    fn auto_remove_message(&self) -> bool {
        true
    }

    fn private_response(&self)-> bool {
        false
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

