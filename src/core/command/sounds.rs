// Internal Dependencies ------------------------------------------------------
use super::super::super::util;
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// Sound Listing --------------------------------------------------------------
pub struct Sounds {
    patterns: Vec<String>
}

impl Sounds {
    pub fn new(args: Vec<&str>) -> Sounds {
        Sounds {
            patterns: args.iter().map(|a| a.to_string()).collect()
        }
    }
}


// Command Implementation -----------------------------------------------------
impl Command for Sounds {

    fn execute(&mut self, _: &mut Handle, server: &mut Server, user: &User) -> CommandResult {

        if self.patterns.is_empty() {
            info!("[{}] [{}] [Command] [Sounds] Sound listing requested.", server, user);
            let mut effects = server.list_effects();
            effects.sort();
            Some(util::list_words("Sound Effects", effects, 100, 4))

        } else {

            info!(
                "[{}] [{}] [Command] [Sounds] Sound listing for pattern \"{}\" requested.",
                server, user, self.patterns.join("\", \"")
            );

            let effects = server.map_effects(&self.patterns[..], true);
            let mut effects: Vec<&str> = effects.iter().map(|effect| {
                effect.get_name()

            }).collect();

            effects.sort();

            let title = format!("Sound Effect matching \"{}\"", self.patterns.join("\", \""));
            Some(util::list_words(title.as_str(), effects, 100, 4))

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

