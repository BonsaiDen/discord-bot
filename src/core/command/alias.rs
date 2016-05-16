// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// Effect Alias Setup ---------------------------------------------------------
pub struct Alias {
    mode: Option<String>,
    name: Option<String>,
    effects: Vec<String>
}


// Interface ------------------------------------------------------------------
impl Alias {
    pub fn new(args: Vec<&str>) -> Alias {
        Alias {
            mode: args.get(0).and_then(|s| Some(s.to_string())),
            name: args.get(1).and_then(|s| Some(s.to_string())),
            effects: args.iter().skip(2).map(|s| s.to_string()).collect()
        }
    }
}

impl Alias {

    fn usage() -> CommandResult {
        Some(vec!["Usage: `!alias add|remove <name> [<effect>, ...]`".to_string()])
    }

    fn add(&self, server: &mut Server, user: &User, exists: bool) -> CommandResult {

        let name = self.name.as_ref().unwrap();
        info!(
            "[{}] [{}] [Command] [Alias] Adding effect alias \"{}\" for \"{}\".",
            server, user, name, self.effects.join("\", \"")
        );

        if exists {
            Some(vec![format!(
                "Alias `{}` already exists on the current server.",
                name
            )])

        } else {
            server.add_effect_alias(name, self.effects.clone());
            Some(vec![format!(
                "Added custom effect alias `{}` for `{}` on the current server.",
                name, self.effects.join("`, `")
            )])
        }

    }

    fn remove(&self, server: &mut Server, user: &User, exists: bool) -> CommandResult {

        let name = self.name.as_ref().unwrap();
        info!(
            "[{}] [{}] [Command] [Alias] Removing effect alias `{}`.",
            server, user, name
        );

        if exists {
            server.remove_effect_alias(name);
            Some(vec![format!(
                "Removed effect alias `{}` from the current server.",
                name
            )])

        } else {
            Some(vec![format!(
                "Alias `{}` does not exists on the current server.",
                name
            )])
        }

    }

}


// Command Implementation -----------------------------------------------------
impl Command for Alias {

    fn execute(&mut self, _: &mut Handle, server: &mut Server, user: &User) -> CommandResult {

        if self.mode.is_none() || self.name.is_none() {
            Alias::usage()

        } else {

            let exists = server.get_aliases().contains_key(self.name.as_ref().unwrap());

            match &self.mode.as_ref().unwrap()[..] {
                "add" => {
                    if self.effects.is_empty() {
                        Alias::usage()

                    } else {
                        self.add(server, user, exists)
                    }
                }
                "remove" => self.remove(server, user, exists),
                _ => Alias::usage()
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

