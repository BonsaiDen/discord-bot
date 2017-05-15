// Discord Dependencies -------------------------------------------------------
use discord::model::UserId;


// External Dependencies ------------------------------------------------------
use diesel;
use diesel::prelude::*;


// Internal Dependencies ------------------------------------------------------
use super::super::Server;
use ::bot::BotConfig;
use ::effect::Effect;
use ::db::models::{Greeting, NewGreeting};
use ::db::schema::greetings::dsl::{server_id, nickname as greeting_nickname};
use ::db::schema::greetings::table as greetingsTable;


// Server Greeting Interface --------------------------------------------------
impl Server {

    pub fn get_greeting(
        &self,
        member_id: &UserId,
        bot_config: &BotConfig

    ) -> Option<Vec<&Effect>> {
        if let Some(member) = self.members.get(member_id) {

            // User specific greeting
            if let Some(greeting) = self._get_greeting(&member.nickname) {
                Some(self.map_effects(
                    &[greeting.effect_name.to_string()],
                    false,
                    bot_config
                ))

            // Default greeting
            } else if let Some(greeting) = self._get_greeting("default") {
                Some(self.map_effects(
                    &[greeting.effect_name.to_string()],
                    false,
                    bot_config
                ))

            } else {
                None
            }

        } else {
            None
        }
    }

    fn _get_greeting(&self, nickname: &str) -> Option<Greeting> {
        greetingsTable.filter(
            server_id.eq(&self.config.table_id)

        ).filter(
            greeting_nickname.eq(nickname)

        ).first(&self.config.connection).ok()
    }

    pub fn has_greeting(&self, nickname: &str) -> bool {
        greetingsTable.filter(
            server_id.eq(&self.config.table_id)

        ).filter(
            greeting_nickname.eq(nickname)

        ).count().get_result(&self.config.connection).unwrap_or(0) > 0
    }

    pub fn add_greeting(&mut self, nickname: &str, effect_name: &str) {
        diesel::insert(&NewGreeting {
            server_id: &self.config.table_id,
            nickname: nickname,
            effect_name: effect_name

        }).into(greetingsTable).execute(&self.config.connection).ok();
    }

    pub fn remove_greeting(&mut self, nickname: &str) {
        diesel::delete(
            greetingsTable.filter(
                server_id.eq(&self.config.table_id)

            ).filter(
                greeting_nickname.eq(nickname)
            )

        ).execute(&self.config.connection).ok();
    }

    pub fn list_greetings(&self) -> Vec<Greeting> {
        greetingsTable.filter(
            server_id.eq(&self.config.table_id)

        ).order(
            greeting_nickname

        ).load::<Greeting>(&self.config.connection).unwrap_or_else(|_| vec![])
    }

}

