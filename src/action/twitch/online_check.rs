// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Discord Dependencies -------------------------------------------------------
use discord::model::{ChannelId, ServerId};


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::EventQueue;
use ::action::{ActionHandler, ActionGroup};


// Stream Online Check Implementation -----------------------------------------
pub struct Action {
    server_id: ServerId
}

impl Action {
    pub fn new(server_id: ServerId) -> Box<Action> {
        Box::new(Action {
            server_id: server_id
        })
    }
}

impl ActionHandler for Action {

    fn run(&mut self, bot: &mut Bot, config: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.server_id) {
            for streamer in server.list_streamers() {

                match super::twitch::get_stream(config, &streamer.twitch_nick) {
                    Ok(stream) => {

                        let is_online = stream.stream_type == "live";
                        if is_online != streamer.is_online {

                            let channel_id: u64 = streamer.channel_id.parse().expect("Invalid channel id!");
                            let channel_id = ChannelId(channel_id);

                            info!("[Twitch] Channel is: {:?}", server.channel_name(&channel_id));
                            if is_online {
                                info!(
                                    "[Twitch] \"{}\" is now online, playing \"{}\" in {}p for {} viewers.",
                                    streamer.twitch_nick,
                                    stream.game,
                                    stream.resolution,
                                    stream.viewers
                                );

                            } else {
                                info!(
                                    "[Twitch] \"{}\" is now offline.",
                                    streamer.twitch_nick
                                );
                            }

                            server.update_streamer_online_state(&streamer.twitch_nick, is_online);

                        } else {
                            info!("[Twitch] No state change for \"{}\"", streamer.twitch_nick);
                        }

                    },
                    Err(_) => warn!(
                        "[Twitch] Failed to query twitch API for \"{}\" on {}.",
                        streamer.twitch_nick,
                        server.name
                    )
                }

            }
        }

        vec![]

    }

}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[OnlineCheckAction]")
    }
}

