// STD Dependencies -----------------------------------------------------------
use std::io::Read;
use std::path::Path;
use std::ascii::AsciiExt;


// Discord Dependencies -------------------------------------------------------
use discord::{ChannelRef};
use discord::model::{Attachment, ChannelId, MessageId, ServerId};


// External Dependencies ------------------------------------------------------
use hyper::Client;
use hyper::header::{Connection, Range, ByteRangeSpec};
use flac::{ByteStream, Stream};
use flac::metadata::StreamInfo;


// Internal Dependencies ------------------------------------------------------
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
                unique_server
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

        } else if let Some(upload) = self.attachments.get(0) {

            if server.is_admin_user(self.author) {

                let path = Path::new(upload.filename.as_str());
                if let Some(name) = get_upload_name(path) {

                    let effect = name.to_ascii_lowercase();
                    let effect_list = server.list_effects().iter().map(|e| {
                        e.to_ascii_lowercase()

                    }).collect::<String>();

                    if effect_list.contains(&effect) {
                        handle.send_message_to_user(&self.author.id, &format!("A effect with the name `{}` already exists.", effect));
                        warn!("[{}] [{}] [Message] Ignored file upload with existing effect name \"{}\".", server, self.author, effect);

                    } else {
                        info!("[{}] [{}] [Message] Verfiying file format of upload \"{}\" ({})...", server, self.author, effect, upload.url);

                        // TODO fetch and return / validate file size
                        if let Some(info) = fetch_flac_info(&upload.url) {
                            if info.sample_rate == 48000 && info.bits_per_sample == 16 {

                                info!("[{}] [{}] [Message] Upload verified as flac, downloading onto server...", server, self.author);
                                handle.send_message_to_user(&self.author.id, "Sound effect upload in progress...");

                                if let Ok(_) = server.download_effect(&effect, &upload.url) {
                                    handle.send_message_to_user(&self.author.id, &format!("The sound effect `{}` was successfully uploaded and is now available!", effect));
                                    warn!("[{}] [{}] [Message] Flac file upload completed for \"{}\" ({}).", server, self.author, effect, upload.url);

                                } else {
                                    handle.send_message_to_user(&self.author.id, &format!("The sound effect `{}` failed to upload, please try again.", effect));
                                    warn!("[{}] [{}] [Message] Flac file upload failed for \"{}\" ({}).", server, self.author, effect, upload.url);
                                }

                            } else {
                                handle.send_message_to_user(&self.author.id, &format!("The uploaded flac file with {}hz and {}bits per sample, does not match the required audio format of 48000hz and 16bits.", info.sample_rate, info.bits_per_sample));
                                warn!("[{}] [{}] [Message] Uploaded flac with {}hz and {}bits per sample, does not match the required audio format of 48000hz and 16bits.", server, self.author, info.sample_rate, info.bits_per_sample);
                            }

                        } else {
                            handle.send_message_to_user(&self.author.id, "The uploaded file is not a valid `flac` file.");
                            warn!("[{}] [{}] [Message] Failed to verify upload as a flac file.", server, self.author);
                        }
                    }

                } else {
                    handle.send_message_to_user(&self.author.id, "The uploaded file has an incorrect filename, please see `!help` for more details.");
                    warn!("[{}] [{}] [Message] Ignored file upload with incorrect filename.", server, self.author);
                }

            } else {
                handle.send_message_to_user(&self.author.id, "Sorry, only admin users can upload sound effects.");
                warn!("[{}] [{}] [Message] Ignored file upload from non-admin user.", server, self.author);
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
fn get_upload_name(path: &Path) -> Option<String> {

    if let Some(filename) = path.file_stem() {

        if filename.len() < 3 {
            None

        } else if let Some(ext) = path.extension() {
            if ext == "flac" {
                if let Some(name) = filename.to_str() {
                    if name.is_ascii() {
                        Some(name.to_string())

                    } else {
                        None
                    }

                } else {
                    None
                }

            } else {
                None
            }

        } else {
            None
        }

    } else {
        None
    }

}

fn fetch_flac_info(url: &str) -> Option<StreamInfo> {

    let client = Client::new();

    if let Ok(mut res) = client.get(url).header(
        Range::Bytes(vec![ByteRangeSpec::FromTo(0, 256)])

    ).header(Connection::close()).send() {
        let mut header = Vec::new();
        if let Ok(_) = res.read_to_end(&mut header) {
            if let Ok(stream) = Stream::<ByteStream>::from_buffer(&header[..]) {
                Some(stream.info())

            } else {
                None
            }

        } else {
            None
        }

    } else {
        None
    }

}

