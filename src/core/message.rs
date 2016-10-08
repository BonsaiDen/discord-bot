// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Discord Dependencies -------------------------------------------------------
use discord::model::{Attachment, MessageId, ChannelId, UserId, ServerId};


// Internal Dependencies ------------------------------------------------------
use ::upload::Upload;
use ::command::Command;


// Message Origin for Server specific Commands --------------------------------
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MessageOrigin {
    PublicServerChannel,
    PrivateServerChannel,
    DirectMessage
}


// Message Kind Abstraction ---------------------------------------------------
pub enum MessageKind {
    Command(Command),
    Upload(Upload)
}


// Message Abstraction --------------------------------------------------------
#[derive(Debug, Copy, Clone)]
pub struct Message {
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub server_id: ServerId,
    pub user_id: UserId,
    pub origin: MessageOrigin
}


// Public Interface -----------------------------------------------------------
impl Message {

    pub fn parse(
        id: MessageId,
        user_id: UserId,
        channel_id: ChannelId,
        content: String,
        attachments: Vec<Attachment>,
        origin: (ServerId, MessageOrigin)

    ) -> Vec<MessageKind> {

        let message = Message {
            id: id,
            channel_id: channel_id,
            server_id: origin.0,
            user_id: user_id,
            origin: origin.1
        };

        info!("{} received", message);

        let mut kinds = Vec::new();
        if content.starts_with('!') {

            let mut split = content.split(' ');
            let command_name = split.next().unwrap_or("!");

            kinds.push(MessageKind::Command(Command::new(
                command_name[1..].to_string(),
                split.map(|s| s.to_string()).collect(),
                message
            )))

        } else if !attachments.is_empty() {
            for attachment in attachments {
                kinds.push(MessageKind::Upload(
                    Upload::new(attachment, message)
                ))
            }
        }

        kinds

    }

}

// Traits ---------------------------------------------------------------------
impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.origin {
            MessageOrigin::PublicServerChannel => {
                write!(
                    f,
                    "[Server Message #{} from #{} in #{}(public) for #{}]",
                    self.id, self.user_id, self.channel_id, self.server_id
                )
            },
            MessageOrigin::PrivateServerChannel => {
                write!(
                    f,
                    "[Server Message #{} from #{} in #{}(private) for #{}]",
                    self.id, self.user_id, self.channel_id, self.server_id
                )
            },
            MessageOrigin::DirectMessage  => {
                write!(
                    f,
                    "[Private Message #{} from #{} in #{}]",
                    self.id, self.user_id, self.channel_id
                )
            }
        }
    }
}


