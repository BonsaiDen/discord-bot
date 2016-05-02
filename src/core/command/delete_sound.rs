// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// Sound Deletion Setup -------------------------------------------------------
pub struct DeleteSound {
    name: Option<String>
}


// Interface ------------------------------------------------------------------
impl DeleteSound {
    pub fn new(args: Vec<&str>) -> DeleteSound {
        DeleteSound {
            name: args.get(0).and_then(|s| Some(s.to_string()))
        }
    }
}

impl DeleteSound {

    fn usage() -> CommandResult {
        Some(vec!["Usage: `!delete <effect>`".to_string()])
    }

}


// Command Implementation -----------------------------------------------------
impl Command for DeleteSound {

    fn execute(&self, _: &mut Handle, server: &mut Server, user: &User) -> CommandResult {

        if self.name.is_none() {
            DeleteSound::usage()

        } else {

            let name = self.name.as_ref().unwrap();
            if server.get_aliases().contains_key(name) {

                info!(
                    "[{}] [{}] [Command] [DeleteSound] Deleting sound effect \"{}\".",
                    server, user, name
                );

                if let Err(err) = server.delete_effect(name) {
                    info!(
                        "[{}] [{}] [Command] [DeleteSound] Failed to delete sound effect \"{}\": {}.",
                        server, user, name, err
                    );
                    Some(vec![format!("Failed to delete sound effect `{}`.", name)])

                } else {
                    info!(
                        "[{}] [{}] [Command] [DeleteSound] Sound effect \"{}\" deleted.",
                        server, user, name
                    );
                    Some(vec![format!("Sound effect `{}` was deleted.", name)])
                }

            } else {
                Some(vec![format!("Sound effect `{}` does not exist.", name)])
            }

        }

    }

    fn requires_admin_user(&self) -> bool {
        true
    }

    fn requires_unique_server(&self) -> bool {
        false
    }

    fn auto_remove_message(&self) -> bool {
        false
    }

    fn private_response(&self)-> bool {
        true
    }

}

