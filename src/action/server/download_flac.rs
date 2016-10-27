// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::{EventQueue, Message};
use ::action::{ActionHandler, ActionGroup, MessageActions};


// Action Implementation ------------------------------------------------------
pub struct Action {
    message: Message,
    effect_name: String,
    upload_url: String,
    uploader: String
}

impl Action {
    pub fn new(
        message: Message,
        effect_name: String,
        upload_url: String,
        uploader: String

    ) -> Box<Action> {
        Box::new(Action {
            message: message,
            effect_name: effect_name,
            upload_url: upload_url,
            uploader: uploader
        })
    }
}

impl ActionHandler for Action {
    fn run(&mut self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {

            if server.has_effect(&self.effect_name) {
                MessageActions::Send::public(
                    &self.message,
                    format!(
                        "A sound effect with the name `{}` already exists on the server.",
                        self.effect_name
                    )
                )

            } else {

                info!("{} Downloading as {}...", self, self.effect_name);

                if let Err(err) = server.download_effect(
                    &self.effect_name,
                    &self.upload_url,
                    &self.uploader
                ) {
                    warn!("{} Download failed: {}", self, err);
                    MessageActions::Send::public(
                        &self.message,
                        format!(
                            "Download of the sound effect `{}` failed, please try again.",
                            self.effect_name
                        )
                    )

                } else {
                    info!("{} Download successful.", self);
                    MessageActions::Send::public(
                        &self.message,
                        format!(
                            "The sound effect was successfully downloaded to the server and is now available as `{}`!",
                            self.effect_name
                        )
                    )
                }

            }

        } else {
            vec![]
        }

    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Action] [DownloadFlac] \"{}\" from {} on Server#{}",
            self.upload_url, self.uploader, self.message.server_id
        )
    }
}

