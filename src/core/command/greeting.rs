// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// User Greeting Setup --------------------------------------------------------
pub struct Greeting {
    mode: Option<String>,
    nickname: Option<String>,
    effect: Option<String>
}


// Interface ------------------------------------------------------------------
impl Greeting {
    pub fn new(args: Vec<&str>) -> Greeting {
        Greeting {
            mode: args.get(0).and_then(|s| Some(s.to_string())),
            nickname: args.get(1).and_then(|s| Some(s.to_string())),
            effect: args.get(2).and_then(|s| Some(s.to_string()))
        }
    }
}

impl Greeting {

    fn usage() -> CommandResult {
        Some(vec!["Usage: `!greeting add|remove <user> [<effect>]`".to_string()])
    }

    fn add(&self, server: &mut Server, user: &User, greeted_user: Option<User>) -> CommandResult {

        let effect = self.effect.as_ref().unwrap();
        let nickname = self.nickname.as_ref().unwrap();
        info!(
            "[{}] [{}] [Command] [Greeting] Adding greeting \"{}\" for user {}.",
            server, user, effect, nickname
        );

        if let Some(_) = greeted_user {
            server.add_user_greeting(nickname, effect);
            Some(vec![format!(
                "Added custom greeting \"{}\" for {} on the current server.",
                effect, nickname
            )])

        } else {
            Some(vec![format!(
                "User {} is not a member of the current server.",
                nickname
            )])
        }

    }

    fn remove(&self, server: &mut Server, user: &User, greeted_user: Option<User>) -> CommandResult {

        let nickname = self.nickname.as_ref().unwrap();
        info!(
            "[{}] [{}] [Command] [Greeting] Removing greeting for user {}.",
            server, user, nickname
        );

        if let Some(_) = greeted_user {
            server.remove_user_greeting(nickname);
            Some(vec![format!(
                "Removed any custom greeting for {} on the current server.",
                nickname
            )])

        } else {
            Some(vec![format!(
                "User {} is not a member of the current server.",
                nickname
            )])
        }

    }

}


// Command Implementation -----------------------------------------------------
impl Command for Greeting {

    fn execute(&mut self, handle: &mut Handle, server: &mut Server, user: &User) -> CommandResult {

        if self.mode.is_none() || self.nickname.is_none() {
            Greeting::usage()

        } else {

            let greeted_user = handle.find_server_user_by_nickname(
                &server.id(), &self.nickname.as_ref().unwrap()
            );

            match &self.mode.as_ref().unwrap()[..] {
                "add" => {
                    if self.effect.is_none() {
                        Greeting::usage()

                    } else {
                        self.add(server, user, greeted_user)
                    }
                }
                "remove" => self.remove(server, user, greeted_user),
                _ => Greeting::usage()
            }

        }

    }

    fn requires_unique_server(&self) -> bool {
        true
    }

    fn auto_remove_message(&self) -> bool {
        true
    }

    fn private_response(&self)-> bool {
        true
    }

}

