// STD Dependencies -----------------------------------------------------------
use std::ffi::OsStr;
use std::path::Path;
use std::ascii::AsciiExt;


// Discord Dependencies -------------------------------------------------------
use discord::ChannelRef;
use discord::model::{Attachment, ChannelId, MessageId, ServerId};


// Internal Dependencies ------------------------------------------------------
use super::super::util;
use super::{command, Handle, Server, User};


// Message Abstraction --------------------------------------------------------
pub struct Message<'a> {
    pub id: &'a MessageId,
    pub server_id: &'a ServerId,
    pub channel_id: &'a ChannelId,
    pub author: &'a User,
    pub content: &'a str,
    pub attachments: Vec<Attachment>,
    pub was_edited: bool
}


// Message Handling -----------------------------------------------------------
impl<'a> Message<'a> {

    pub fn handle(
        &self,
        handle: &mut Handle,
        server: &mut Server,
        unique_server: bool
    ) {

        if self.author.is_bot {
            debug!("[Message] Ignored message from bot.");
            return;

        } else {
            self.log(handle, server);
        }

        if self.content.starts_with('!') {

            let mut split = self.content.split(' ');
            let name = split.next().unwrap_or("!");
            let command = command::from_args(
                &name[1..],
                split.collect(),
                unique_server,
                server.is_admin_user(self.author)
            );

            if let Some(responses) = command.execute(handle, server, self.author) {
                for response in responses {
                    if command.private_response() {
                        handle.send_message_to_user(&self.author.id, &response);

                    } else {
                        handle.send_message_to_channel(&self.channel_id, &response);
                    }
                }
            }

            if command.auto_remove_message() {
                if handle.delete_message(&self) {
                    info!("[{}] [{}] [Message] Deleted message #{}.", server, self.author, self.id.0);

                } else {
                    warn!("[{}] [{}] [Message] Cannot delete message in private channel.", server, self.author);
                }
            }

        } else if !self.attachments.is_empty() {

            if !server.is_upload_user(self.author) {
                handle.send_message_to_user(&self.author.id, "Sorry, only whitelisted users can upload sound effects.");
                warn!("[{}] [{}] [Message] Ignored file upload from non-whitelisted user.", server, self.author);

            } else if let Some(ChannelRef::Private(_)) = handle.find_channel_by_id(&self.channel_id) {
                // TODO Need to check if channel recipient is bot?
                for attachment in &self.attachments {
                    self.handle_attachment(handle, server, attachment);
                }

            } else {
                warn!("[{}] [{}] [Message] Ignored file upload from public channel.", server, self.author);
            }

        }

    }

    fn handle_attachment(&self, handle: &mut Handle, server: &mut Server, attachment: &Attachment) {
        match verify_upload(&server, attachment) {
            Ok((effect, url)) => {

                info!("[{}] [{}] [Message] Upload verified as flac firl with correct format, now downloading onto server...", server, self.author);
                handle.send_message_to_user(&self.author.id, "Sound effect upload in progress...");

                if let Err(err) = server.download_effect(&effect, &url) {
                    handle.send_message_to_user(&self.author.id, &format!("The sound effect `{}` failed to upload, please try again.", effect));
                    warn!("[{}] [{}] [Message] Sound effect upload failed for \"{}\" ({}): {}.", server, self.author, effect, url, err);

                } else {
                    handle.send_message_to_user(&self.author.id, &format!("The sound effect `{}` was successfully uploaded and is now available!", effect));
                    info!("[{}] [{}] [Message] Sound effect upload completed for \"{}\" ({}).", server, self.author, effect, url);
                }

            }
            Err(err) => {
                handle.send_message_to_user(&self.author.id, &err);
                warn!("[{}] [{}] [Message] {}", server, self.author, err);
            }
        }
    }

    fn log(&self, handle: &mut Handle, server: &mut Server) {
        match handle.find_channel_by_id(&self.channel_id) {

            Some(ChannelRef::Public(_, channel)) => {
                info!(
                    "[{}#{}] [{}] [Message]: {}",
                    server, channel.name,
                    self.author.nickname,
                    self.content
                );
            }

            Some(ChannelRef::Private(channel)) => {

                if self.author.name == channel.recipient.name {
                    info!(
                        "[{}] [{}] [Message] [Private]: {}",
                        server,
                        self.author.nickname,
                        self.content
                    );

                } else {
                    info!(
                        "[{}] [Message] [Private] To [{}#{}]: {}",
                        server,
                        channel.recipient.name, channel.recipient.discriminator,
                        self.content
                    );
                }

            }

            None => info!(
                "[{}] [{}] [Message] [Unknown Channel]: {}",
                server,
                self.author.nickname,
                self.content
            )
        }
    }

}


// Helpers --------------------------------------------------------------------
fn verify_upload<'a>(server: &Server, upload: &'a Attachment) -> Result<(String, &'a str), String> {

    let path = Path::new(upload.filename.as_str());
    if let Some(name) = verify_flac_name(path) {

        let effect_list = server.list_effects().iter().map(|e| {
            e.to_ascii_lowercase()

        }).collect::<String>();

        let effect = name.to_ascii_lowercase();
        if effect_list.contains(&effect) {
            Err(format!("A effect with the name `{}` already exists.", effect))

        } else if let Ok((length, info)) = util::retrieve_flac_info(&upload.url) {
            if length > 2048 * 1024 {
                Err(format!("The uploaded flac file size of {} bytes may not exceed 2 MiB.", length))

            } else if info.sample_rate != 48000 || info.bits_per_sample != 16 {
                Err(format!("The uploaded flac file with {}hz and {}bits per sample, does not match the required audio format of 48000hz and 16bits.", info.sample_rate, info.bits_per_sample))

            } else {
                Ok((effect, &upload.url))
            }

        } else {
            Err("The uploaded file is not a valid `flac` file.".to_string())
        }

    } else {
        Err("The uploaded file has an incorrect filename, please see `!help` for more details.".to_string())
    }

}

fn verify_flac_name(path: &Path) -> Option<String> {

    let name = path.file_stem().unwrap_or_else(|| OsStr::new("")).to_str().unwrap_or("");
    let ext = path.extension().unwrap_or_else(|| OsStr::new(""));

    if name.is_ascii() && name.len() >= 3 && ext == "flac" {
        Some(name.to_string())

    } else {
        None
    }

}

