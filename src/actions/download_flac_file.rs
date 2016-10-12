// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::message::Message;
use ::core::event::EventQueue;
use ::actions::SendPublicMessage;
use ::actions::{Action, ActionGroup};


// Action Implementation ------------------------------------------------------
pub struct DownloadFlacFile {
    message: Message,
    effect_name: String,
    upload_url: String,
    uploader: String
}

impl DownloadFlacFile {
    pub fn new(
        message: Message,
        effect_name: String,
        upload_url: String,
        uploader: String

    ) -> Box<DownloadFlacFile> {
        Box::new(DownloadFlacFile {
            message: message,
            effect_name: effect_name,
            upload_url: upload_url,
            uploader: uploader
        })
    }
}

impl Action for DownloadFlacFile {
    fn run(&self, bot: &mut Bot, _: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {

            if server.has_effect(&self.effect_name) {
                vec![SendPublicMessage::new(
                    &self.message,
                    format!(
                        "A sound effect with the name `{}` already exists on the server.",
                        self.effect_name
                    )
                )]

            } else {

                info!("{} Downloading as {}...", self, self.effect_name);

                if let Err(err) = server.download_effect(
                    &self.effect_name,
                    &self.upload_url,
                    &self.uploader
                ) {
                    warn!("{} Download failed: {}", self, err);
                    vec![SendPublicMessage::new(
                        &self.message,
                        format!(
                            "Download of the sound effect `{}` failed, please try again.",
                            self.effect_name
                        )
                    )]

                } else {
                    info!("{} Download successful.", self);
                    vec![SendPublicMessage::new(
                        &self.message,
                        format!(
                            "The sound effect was successfully downloaded to the server and is now available as `{}`!",
                            self.effect_name
                        )
                    )]
                }

            }

        } else {
            vec![]
        }

    }
}

impl fmt::Display for DownloadFlacFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Action] [DownloadFlacFile] \"{}\" from {} on Server#{}",
            self.upload_url, self.uploader, self.message.server_id
        )
    }
}

