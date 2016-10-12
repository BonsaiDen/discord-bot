// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Discord Dependencies -------------------------------------------------------
use discord::model::{ChannelId, UserId};


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::message::Message;
use ::core::event::EventQueue;
use ::actions::{Action, ActionGroup};


// Action Implementation ------------------------------------------------------
pub struct SendMessage {
    user_id: Option<UserId>,
    channel_id: Option<ChannelId>,
    content: String
}

impl SendMessage {

    pub fn private(message: &Message, content: String) -> Box<SendMessage> {
        Box::new(SendMessage {
            user_id: Some(message.user_id),
            channel_id: None,
            content: content
        })
    }

    pub fn public(message: &Message, content: String) -> Box<SendMessage> {
        Box::new(SendMessage {
            user_id: None,
            channel_id: Some(message.channel_id),
            content: content
        })
    }

}

impl Action for SendMessage {
    fn run(&self, _: &mut Bot, _: &BotConfig, queue: &mut EventQueue) -> ActionGroup {

        if let Some(user_id) = self.user_id.as_ref() {
            queue.send_message_to_user(user_id, self.content.clone());

        } else if let Some(channel_id) = self.channel_id.as_ref() {
            queue.send_message_to_channel(channel_id, self.content.clone());
        }

        vec![]

    }
}

impl fmt::Display for SendMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(user_id) = self.user_id {
            write!(f, "[Action] [SendMessage] To User#{}", user_id)

        } else if let Some(channel_id) = self.channel_id {
            write!(f, "[Action] [SendMessage] To Channel#{}", channel_id)

        } else {
            unreachable!()
        }
    }
}

