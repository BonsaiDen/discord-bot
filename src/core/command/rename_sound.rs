// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// Sound Renaming Setup -------------------------------------------------------
pub struct RenameSound {
    old_name: Option<String>,
    new_name: Option<String>
}


// Interface ------------------------------------------------------------------
impl RenameSound {
    pub fn new(args: Vec<&str>) -> RenameSound {
        RenameSound {
            old_name: args.get(0).and_then(|s| Some(s.to_string())),
            new_name: args.get(1).and_then(|s| Some(s.to_string()))
        }
    }
}

impl RenameSound {

    fn usage() -> CommandResult {
        Some(vec!["Usage: `!rename <old_effect> <new_effect>`".to_string()])
    }

}


// Command Implementation -----------------------------------------------------
impl Command for RenameSound {

    fn execute(&mut self, _: &mut Handle, server: &mut Server, user: &User) -> CommandResult {

        if self.old_name.is_none() || self.new_name.is_none() {
            RenameSound::usage()

        } else {

            let old_name = self.old_name.as_ref().unwrap();
            if server.list_effects().contains(&old_name.as_str()) {

                let new_name = self.new_name.as_ref().unwrap();
                if !server.list_effects().contains(&new_name.as_str()) {

                    info!(
                        "[{}] [{}] [Command] [RenameSound] Renaming sound effect \"{}\" to \"{}\".",
                        server, user, old_name, new_name
                    );

                    if let Err(err) = server.rename_effect(old_name.as_str(), new_name.as_str()) {
                        info!(
                            "[{}] [{}] [Command] [RenameSound] Failed to rename sound effect \"{}\" to \"{}\": {}.",
                            server, user, old_name, new_name, err
                        );
                        Some(vec![format!("Failed to renaem sound effect `{}` to `{}`.", old_name, new_name)])

                    } else {
                        info!(
                            "[{}] [{}] [Command] [RenameSound] Sound effect \"{}\" renamed to \"{}\".",
                            server, user, old_name, new_name
                        );
                        Some(vec![format!("Sound effect `{}` was renamed to `{}`.", old_name, new_name)])
                    }


                } else {
                    Some(vec![format!("Sound effect `{}` already exist, cannot rename.", new_name)])
                }

            } else {
                Some(vec![format!("Sound effect `{}` does not exist, cannot rename.", old_name)])
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
        false
    }

}

