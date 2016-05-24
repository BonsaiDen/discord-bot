// Internal Dependencies ------------------------------------------------------
use super::super::{Handle, Server, User};
use super::{Command, CommandResult};


// Audio Recording -- ---------------------------------------------------------
pub struct Record {
    mode: Option<String>,
    private_response: bool
}


impl Record {

    pub fn new(args: Vec<&str>) -> Record {
        Record {
            mode: args.get(0).and_then(|s| Some(s.to_string())),
            private_response: false
        }
    }

    fn start(&mut self, handle: &mut Handle, server: &mut Server, user: &User) -> CommandResult {

        if let Some(channel_id) = handle.find_voice_channel_id_for_user(&user.id) {
            if let Some(filename) = server.start_recording(handle, channel_id) {
                info!("[{}] [{}] [Command] [Record] Audio recording started ({}).", server, user, filename);
                self.private_response = false;
                Some(vec![
                    format!("{} has started recording audio in this channel ({}).", user.nickname, filename)
                ])

            } else {
                self.private_response = true;
                Some(vec![
                    format!("Audio is already being recorded in this channel.")
                ])
            }

        } else {
            self.private_response = true;
            Some(vec![
                format!("You must be in a voice channel in order to record audio.")
            ])
        }

    }

    fn stop(&mut self, server: &mut Server, user: &User) -> CommandResult {
        if let Some(filename) = server.stop_recording() {
            info!("[{}] [{}] [Command] [Record] Audio recording stopped ({}).", server, user, filename);
            self.private_response = false;
            Some(vec![
                format!("{} has stopped recording audio in this channel ({}).", user.nickname, filename)
            ])

        } else {
            self.private_response = true;
            Some(vec![
                format!("Audio is currently not being recorded in this channel.")
            ])
        }
    }

    fn usage(&mut self) -> CommandResult {
        self.private_response = true;
        Some(vec!["Usage: `!record start|stop`".to_string()])
    }

}


// Command Implementation -----------------------------------------------------
impl Command for Record {

    fn execute(&mut self, handle: &mut Handle, server: &mut Server, user: &User) -> CommandResult {
        let arg = self.mode.clone().unwrap_or_else(|| "".to_string());
        match &arg[..] {
            "start" => self.start(handle, server, user),
            "stop" => self.stop(server, user),
             _ => self.usage()
        }
    }

    fn requires_unique_server(&self) -> bool {
        true
    }

    fn auto_remove_message(&self) -> bool {
        true
    }

    fn private_response(&self)-> bool {
        self.private_response
    }

}

