// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::io::Read;
use std::ffi::OsStr;
use std::path::Path;
use std::ascii::AsciiExt;


// Discord Dependencies -------------------------------------------------------
use discord::model::Attachment;


// External Dependencies ------------------------------------------------------
use hyper::Client;
use hyper::header::{Connection, Range, ByteRangeSpec, ContentLength};
use flac::{ByteStream, Stream};


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::core::{Member, Message, MessageOrigin, Server};
use ::action::{ActionGroup, ServerActions, MessageActions};


// Flac File Information ------------------------------------------------------
#[derive(Debug)]
pub struct FlacInfo {
    file_size: u64,
    sample_rate: u32,
    bits_per_sample: u8
}

impl fmt::Display for FlacInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, "{} bytes, {}hz @ {}bit",
            self.file_size, self.sample_rate, self.bits_per_sample
        )
    }
}


// Upload Abstraction ---------------------------------------------------------
#[derive(Debug)]
pub struct Upload {
    pub name: String,
    url: String,
    flac_info: Option<FlacInfo>,
    pub message: Message
}


// Public Interface -----------------------------------------------------------
impl Upload {

    pub fn new(
        attachment: Attachment,
        message: Message

    ) -> Upload {

        let path = Path::new(attachment.filename.as_str());
        let name = os_str_to_string(path.file_stem()).replace(".", "_");
        let ext = os_str_to_string(path.extension());
        let valid_name = name.is_ascii() && name.len() >= 2 && ext == "flac";

        // TODO extract upload comments and use it as the transcription
        Upload {
            name: name.to_string(),
            url: attachment.url.to_string(),
            flac_info: if valid_name {
                retrieve_flac_info(attachment.url).ok()

            } else {
                None
            },
            message: message
        }

    }

    pub fn process(
        self,
        _: &Server,
        member: &Member,
        config: &BotConfig

    ) -> ActionGroup {

        if !member.is_uploader {
            vec![MessageActions::Send::private(
                &self.message,
                "Only whitelisted users can upload sound effects.".to_string()
            )]

        } else if self.message.origin == MessageOrigin::DirectMessage {
            vec![MessageActions::Send::private(
                &self.message,
                "FLAC uploads require a unique server as their target.
                Since you are a member of at least two bot-enabled servers,
                the command cannot be invoked from a private channel.
                Please re-issue the command from a public channels of the target server.".to_string()
            )]

        } else if let Some(flac_info) = self.flac_info {
            if flac_info.file_size > config.flac_max_file_size {
                vec![MessageActions::Send::private(
                    &self.message,
                    "Uploaded FLAC file exceeds 2 MiB.".to_string()
                )]

            } else if flac_info.sample_rate != config.flac_sample_rate {
                vec![MessageActions::Send::private(
                    &self.message,
                    "Uploaded FLAC file does not have a valid sample rate of 48000hz.".to_string()
                )]

            } else if flac_info.bits_per_sample != config.flac_bits_per_sample {
                vec![MessageActions::Send::private(
                    &self.message,
                    "Uploaded FLAC file does not feature 16 bits per sample.".to_string()
                )]

            } else {
                vec![
                    MessageActions::Send::public(
                        &self.message,
                        "FLAC download to server started...".to_string()
                    ),
                    ServerActions::DownloadFlac::new(
                        self.message,
                        self.name,
                        self.url,
                        member.nickname.clone()
                    )
                ]
            }

        } else {
            // Ignore non-FLAC uploads
            vec![]
        }

    }

}


// Traits  --------------------------------------------------------------------
impl fmt::Display for Upload {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref flac) = self.flac_info {
            write!(
                f, "[Flac upload \"{}\" with {} from user #{} on server #{}]",
                self.name, flac, self.message.user_id, self.message.server_id
            )
        } else {

            write!(
                f, "[Unsupported upload \"{}\" from user #{} on server #{}]",
                self.name, self.message.user_id, self.message.server_id
            )
        }
    }
}


// Helpers --------------------------------------------------------------------
fn os_str_to_string(os_str: Option<&OsStr>) -> String {
    os_str.unwrap_or_else(|| {
        OsStr::new("")

    }).to_string_lossy().to_ascii_lowercase()
}


fn retrieve_flac_info(url: String) -> Result<FlacInfo, String> {

    let client = Client::new();
    client.get(&url)
        .header(Range::Bytes(vec![ByteRangeSpec::FromTo(0, 256)]))
        .header(Connection::close())
        .send()
        .map_err(|err| err.to_string())
        .and_then(|mut resp| {
            let length = resp.headers.get::<ContentLength>().map_or(0, |l| l.0);
            let mut header = Vec::new();
            resp.read_to_end(&mut header)
                .map_err(|err| err.to_string())
                .map(|_| (length, header))
        })
        .and_then(|(length, header)| {
            Stream::<ByteStream>::from_buffer(&header[..])
                .map_err(|_| "Failed to parse flac header.".to_string())
                .map(|stream| {
                    let stream_info = stream.info();
                    FlacInfo {
                        file_size: length,
                        sample_rate: stream_info.sample_rate,
                        bits_per_sample: stream_info.bits_per_sample
                    }
                })
        })

}

