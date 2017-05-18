// STD Dependencies -----------------------------------------------------------
use std::fmt;


// External Dependencies ------------------------------------------------------
use chrono;
use rayon::iter::{ParallelIterator, IntoParallelIterator};


// Discord Dependencies -------------------------------------------------------
use discord::model::{ChannelId, ServerId};


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::EventQueue;
use ::action::{ActionHandler, ActionGroup, MessageActions};
use ::db::models::Streamer;


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

            // Run HTTP requests in parallel
            let states: Vec<(Streamer, Option<bool>, ActionGroup)> = server.list_streamers().into_par_iter().map(|streamer| {
                check_stream(config, streamer)

            }).collect();

            // Then check results in the bot thread
            states.into_iter().flat_map(|(streamer, state, actions)| {

                // Update last known streaming state in database
                if let Some(state) = state {
                    server.update_streamer_online_state(
                        &streamer.twitch_nick,
                        state
                    );
                }

                actions

            }).collect()

        } else {
            vec![]
        }
    }

}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[OnlineCheckAction]")
    }
}

fn check_stream(config: &BotConfig, streamer: Streamer) -> (Streamer, Option<bool>, ActionGroup) {
    match super::stream::get_stream(config, &streamer.twitch_nick) {
        Ok(stream) => {

            let is_online = stream.stream_type == "live";
            if is_online != streamer.is_online {

                let channel_id: u64 = streamer.channel_id.parse().expect("Invalid channel id!");
                let channel_id = ChannelId(channel_id);

                if is_online {
                    info!(
                        "[Twitch] \"{}\" is now online, playing \"{}\" in {}p for {} viewers.",
                        streamer.twitch_nick,
                        stream.game,
                        stream.resolution,
                        stream.viewers
                    );

                    // Wait at least 60 seconds between every online announcement
                    let now = chrono::UTC::now().timestamp() as i32;
                    let actions: ActionGroup = if now > streamer.last_online + 60 {
                        vec![
                            MessageActions::Send::single_public_channel(&channel_id, format!(
                                "Twitch streamer **{}** is now online, playing **{}** in {}p for {} viewers!",
                                streamer.twitch_nick,
                                stream.game,
                                stream.resolution,
                                stream.viewers
                            )),
                            MessageActions::Send::single_public_channel(&channel_id, format!(
                                "https://twitch.tv/{}",
                                streamer.twitch_nick,
                            ))
                        ]

                    } else {
                        vec![]
                    };
                    (streamer, Some(true), actions)

                } else {
                    info!( "[Twitch] \"{}\" is now offline.", streamer.twitch_nick);
                    (streamer, Some(false), vec![])
                }

            } else {
                info!("[Twitch] No state change for \"{}\"", streamer.twitch_nick);
                (streamer, None, vec![])
            }

        },
        Err(_) => {
            warn!(
                "[Twitch] Failed to query twitch API for \"{}\".",
                streamer.twitch_nick
            );
            (streamer, None, vec![])
        }
    }
}

