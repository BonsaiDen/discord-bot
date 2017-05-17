// External Dependencies ------------------------------------------------------
use diesel;
use diesel::prelude::*;


// Discord Dependencies -------------------------------------------------------
use discord::model::ChannelId;


// Internal Dependencies ------------------------------------------------------
use super::super::Server;
use ::db::models::{Streamer, NewStreamer};
use ::db::schema::streamers::dsl::{server_id, twitch_nick, is_online};
use ::db::schema::streamers::table as streamerTable;


// Server Streamer Interface --------------------------------------------------
impl Server {

    pub fn has_streamer(&self, name: &str) -> bool {
        streamerTable.filter(
            server_id.eq(&self.config.table_id)

        ).filter(
            twitch_nick.eq(name)

        ).count().get_result(&self.config.connection).unwrap_or(0) > 0
    }

    pub fn add_streamer(&mut self, name: &str, channel_id: ChannelId) {
        diesel::insert(&NewStreamer {
            server_id: &self.config.table_id,
            channel_id: channel_id.to_string(),
            twitch_nick: name,
            is_online: false

        }).into(streamerTable).execute(&self.config.connection).ok();
    }

    pub fn remove_streamer(&mut self, name: &str) {
        diesel::delete(
            streamerTable.filter(
                server_id.eq(&self.config.table_id)

            ).filter(
                twitch_nick.eq(name)
            )

        ).execute(&self.config.connection).ok();
    }

    pub fn update_streamer_online_state(&self, name: &str, set_online: bool) -> bool {
        if self.has_streamer(name) {
            diesel::update(
                streamerTable.filter(
                    server_id.eq(&self.config.table_id)

                ).filter(
                    twitch_nick.eq(name)
                )

            ).set(is_online.eq(set_online)).execute(
                &self.config.connection

            ).ok();
            true

        } else {
            false
        }
    }

    pub fn list_streamers(&self) -> Vec<Streamer> {
        streamerTable.filter(server_id.eq(&self.config.table_id))
                     .load::<Streamer>(&self.config.connection)
                     .unwrap_or_else(|_| vec![])
    }

}

