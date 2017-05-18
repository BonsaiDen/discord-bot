// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::{EventQueue, Message};
use ::action::{ActionHandler, ActionGroup, MessageActions};


// Action Implementation ------------------------------------------------------
pub struct Action {
    message: Message,
    name: String,
    channel_name: String
}

impl Action {
    pub fn new(message: Message, name: String, channel_name: String) -> Box<Action> {
        Box::new(Action {
            message: message,
            name: name,
            channel_name: channel_name
        })
    }
}

impl ActionHandler for Action {
    fn run(&mut self, bot: &mut Bot, config: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {

            if let Some(channel_id) = server.get_channel_id(&self.channel_name) {

                if let Ok(channel) = super::stream::get_channel(config, &self.name) {
                    server.add_streamer(&self.name, channel_id);
                    MessageActions::Send::private(&self.message, format!(
                        "Twitch streamer **{}** ({}) is now being watched on {}, with notifcations send to **#{}**.",
                        channel.display_name,
                        channel.url,
                        server.name,
                        self.channel_name
                    ))

                } else {
                    MessageActions::Send::private(&self.message, format!(
                        "Twitch streamer `{}` is was not found on twitch.tv!",
                        self.name
                    ))
                }

            } else {
                MessageActions::Send::private(&self.message, format!(
                    "A channel named \"{}\" was not found on {}.",
                    self.channel_name, server.name
                ))
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
            "[Action] [AddStreamer] {}",
            self.name
        )
    }
}

