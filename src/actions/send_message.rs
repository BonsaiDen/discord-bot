// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Discord Dependencies -------------------------------------------------------
use discord::model::{ChannelId, UserId};


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::message::Message;
use ::core::event::EventQueue;
use ::actions::{Action, ActionGroup};


// Private Message Sending Actions --------------------------------------------
pub struct SendPrivateMessage {
    user_id: UserId,
    content: String
}

impl SendPrivateMessage {
    pub fn new(message: &Message, content: String) -> Box<SendPrivateMessage> {
        Box::new(SendPrivateMessage {
            user_id: message.user_id,
            content: content
        })
    }
}

impl Action for SendPrivateMessage {
    fn run(&self, _: &mut Bot, _: &BotConfig, queue: &mut EventQueue) -> ActionGroup {
        info!("{} Sending...", self);
        queue.send_message_to_user(&self.user_id, self.content.clone());
        vec![]
    }
}

impl fmt::Display for SendPrivateMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Action] [SendPrivateMessage] To User#{}",
            self.user_id
        )
    }
}


// Public Message Sending Actions ---------------------------------------------
pub struct SendPublicMessage {
    channel_id: ChannelId,
    content: String
}

impl SendPublicMessage {
    pub fn new(message: &Message, content: String) -> Box<SendPublicMessage> {
        Box::new(SendPublicMessage {
            channel_id: message.channel_id,
            content: content
        })
    }
}

impl Action for SendPublicMessage {
    fn run(&self, _: &mut Bot, _: &BotConfig, queue: &mut EventQueue) -> ActionGroup {
        info!("{} Sending...", self);
        queue.send_message_to_channel(&self.channel_id, self.content.clone());
        vec![]
    }
}

impl fmt::Display for SendPublicMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Action] [SendPublicMessage] To Channel#{}",
            self.channel_id
        )
    }
}

