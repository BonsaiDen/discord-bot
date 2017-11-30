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
use ::core::{Member, Message};
use ::action::{ActionGroup, ServerActions, MessageActions};


// Upload File Information ----------------------------------------------------
#[derive(Debug)]
enum FileInfo {
    Flac {
        file_size: u64,
        sample_rate: u32,
        bits_per_sample: u8
    },
    Error(String),
    Text
}


// Upload Abstraction ---------------------------------------------------------
#[derive(Debug)]
pub struct Upload {
    pub name: String,
    message: Message,
    url: String,
    info: Option<FileInfo>
}


// Public Interface -----------------------------------------------------------
impl Upload {

    pub fn from_message(
        attachment: Attachment,
        message: Message

    ) -> Upload {

        let path = Path::new(attachment.filename.as_str());
        let name = os_str_to_string(path.file_stem()).replace(".", "_");
        let ext = os_str_to_string(path.extension());
        info!("[Upload] [Message] {:?}.{:?}", name, ext);

        Upload {
            name: name.to_string(),
            url: attachment.url.to_string(),
            info: if name.is_ascii() && name.len() >= 2 && ext == "flac" {
                match retrieve_flac_info(attachment.url.as_str()) {
                    Ok(info) => Some(info),
                    Err(err) => Some(FileInfo::Error(err))
                }

            } else if name.is_ascii() && name.len() >= 2 && ext == "txt" {
                Some(FileInfo::Text)

            } else {
                None
            },
            message: message
        }

    }

    pub fn process(
        self,
        member: &Member,
        config: &BotConfig

    ) -> ActionGroup {

        if !member.is_uploader {
            MessageActions::Send::private(
                &self.message,
                "Only white listed users can upload sound effects.".to_string()
            )

        } else if !self.message.has_unique_server() {
            MessageActions::Send::private(
                &self.message,
                "FLAC uploads require a unique server as their target.
                Since you are a member of at least two bot-enabled servers,
                the command cannot be invoked from a private channel.
                Please re-issue the command from a public channels of the target server.".to_string()
            )

        } else if let Some(FileInfo::Flac {
            file_size,
            sample_rate,
            bits_per_sample

        }) = self.info {
            if file_size > config.flac_max_file_size {
                MessageActions::Send::private(
                    &self.message,
                    "Uploaded FLAC file exceeds 2 MiB.".to_string()
                )

            } else if sample_rate != config.flac_sample_rate {
                MessageActions::Send::private(
                    &self.message,
                    "Uploaded FLAC file does not have a valid sample rate of 48000hz.".to_string()
                )

            } else if bits_per_sample != config.flac_bits_per_sample {
                MessageActions::Send::private(
                    &self.message,
                    "Uploaded FLAC file does not feature 16 bits per sample.".to_string()
                )

            } else {
                vec![
                    MessageActions::Send::single_public(
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

        } else if let Some(FileInfo::Error(message)) = self.info {
            vec![
                MessageActions::Send::single_public(
                    &self.message,
                    format!("Failed to parse uploaded file: {}", message)
                )
            ]

        } else if let Some(FileInfo::Text) = self.info {
            vec![
                MessageActions::Send::single_public(
                    &self.message,
                    "Transcript download to server started...".to_string()
                ),
                ServerActions::DownloadTranscript::new(
                    self.message,
                    self.name,
                    self.url
                )
            ]

        } else {
            vec![]
        }

    }

}


// Traits  --------------------------------------------------------------------
impl fmt::Display for Upload {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(FileInfo::Flac { .. }) = self.info {
            write!(
                f, "[FLAC upload \"{}\" from user #{} on server #{}]",
                self.name, self.message.user_id, self.message.server_id
            )

        } else if let Some(FileInfo::Text) = self.info {
            write!(
                f, "[Text upload \"{}\" from user #{} on server #{}]",
                self.name, self.message.user_id, self.message.server_id
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


fn retrieve_flac_info(url: &str) -> Result<FileInfo, String> {

    let client = Client::new();
    client.get(url)
        .header(Range::Bytes(vec![ByteRangeSpec::FromTo(0, 512)]))
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
            println!("[Upload] [FlacHeader] {:?}", header);
            Stream::<ByteStream>::from_buffer(&header[..])
                .map_err(|err| format!("Failed to parse FLAC header: {:?}", err))
                .map(|stream| {
                    let stream_info = stream.info();
                    FileInfo::Flac {
                        file_size: length,
                        sample_rate: stream_info.sample_rate,
                        bits_per_sample: stream_info.bits_per_sample
                    }
                })
        })

}

