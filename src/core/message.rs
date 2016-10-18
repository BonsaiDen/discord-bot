// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Discord Dependencies -------------------------------------------------------
use discord::model::{Attachment, MessageId, ChannelId, UserId, ServerId};


// Internal Dependencies ------------------------------------------------------
use ::upload::Upload;
use ::command::Command;


// Message Content Abstraction ------------------------------------------------
pub enum MessageContent {
    Command(Command),
    Upload(Upload)
}


// Message Abstraction --------------------------------------------------------
#[derive(Debug, Copy, Clone)]
pub struct Message {
    pub id: MessageId,
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub server_id: ServerId,
    server_is_unique: bool
}


// Public Interface -----------------------------------------------------------
impl Message {

    pub fn from_parts(
        id: MessageId,
        user_id: UserId,
        channel_id: ChannelId,
        server_id: ServerId,
        server_is_unique: bool

    ) -> Message {
        Message {
            id: id,
            user_id: user_id,
            channel_id: channel_id,
            server_id: server_id,
            server_is_unique: server_is_unique
        }
    }

    pub fn parse_contents(
        self,
        content: String,
        attachments: Vec<Attachment>

    ) -> Vec<MessageContent> {

        info!("{} parsing contents...", self);

        if content.starts_with('!') {

            let mut split = content.split(' ');
            let command_name = split.next().unwrap_or("!");

            vec![MessageContent::Command(Command::new(
                command_name[1..].to_string(),
                split.map(|s| s.to_string()).collect(),
                self
            ))]

        } else {
            attachments.into_iter().map(|attachment| {
                MessageContent::Upload(Upload::new(attachment, self))

            }).collect()
        }

    }

    pub fn has_unique_server(&self) -> bool {
        self.server_is_unique
    }

}

// Traits ---------------------------------------------------------------------
impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.has_unique_server() {
            write!(
                f,
                "[Server Message #{} from #{} in #{}(public) for #{}]",
                self.id, self.user_id, self.channel_id, self.server_id
            )

        } else {
            write!(
                f,
                "[Other Message #{} from #{} in #{}]",
                self.id, self.user_id, self.channel_id
            )
        }
    }
}


