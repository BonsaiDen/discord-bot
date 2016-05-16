// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// Audio Recording -- ---------------------------------------------------------
pub struct Record {
    mode: Option<String>
}


impl Record {

    pub fn new(args: Vec<&str>) -> Record {
        Record {
            mode: args.get(0).and_then(|s| Some(s.to_string()))
        }
    }

    fn start(&self, server: &mut Server, user: &User) -> CommandResult {
        if server.start_recording() {
            info!("[{}] [{}] [Command] [Record] Starting audio recording...", server, user);
            Some(vec![
                format!("{} has started recording audio in this channel.", user.nickname)
            ])

        } else {
            Some(vec![
                format!("Audio is already being recorded in this channel.")
            ])
        }
    }

    fn stop(&self, server: &mut Server, user: &User) -> CommandResult {
        if server.stop_recording() {
            info!("[{}] [{}] [Command] [Record] Stopping audio recording...", server, user);
            Some(vec![
                format!("{} has started stopped audio in this channel.", user.nickname)
            ])

        } else {
            Some(vec![
                format!("Audio is currently not being recorded in this channel.")
            ])
        }
    }

    fn usage(&self) -> CommandResult {
        Some(vec!["Usage: `!record start|stop`".to_string()])
    }

}


// Command Implementation -----------------------------------------------------
impl Command for Record {

    fn execute(&self, _: &mut Handle, server: &mut Server, user: &User) -> CommandResult {

        if let Some(arg) = self.mode.as_ref() {
            match &arg[..] {
                "start" => self.start(server, user),
                "stop" => self.stop(server, user),
                 _ => self.usage()
            }

        } else {
            self.usage()
        }

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

