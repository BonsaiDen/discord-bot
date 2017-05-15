// External Dependencies ------------------------------------------------------
use diesel;
use diesel::prelude::*;


// Internal Dependencies ------------------------------------------------------
use super::super::Server;
use ::db::models::User;
use ::db::schema::users::dsl::{server_id, nickname as user_nickname, is_banned};
use ::db::schema::users::table as userTable;


// Server Ban Interface -------------------------------------------------------
impl Server {

    pub fn list_bans(&self) -> Vec<User> {
        userTable.filter(
            server_id.eq(&self.config.table_id)

        ).filter(is_banned.eq(true))
         .order(user_nickname)
         .load::<User>(&self.config.connection)
         .unwrap_or_else(|_| vec![])
    }

    pub fn add_ban(&mut self, nickname: &str) -> bool {
        ::db::create_user_if_not_exists(&self.config, nickname).ok();
        self.update_ban_user(nickname, true)
    }

    pub fn remove_ban(&mut self, nickname: &str) -> bool {
        self.update_ban_user(nickname, false)
    }

    fn update_ban_user(&self, nickname: &str, set_banned: bool) -> bool {
        if ::db::user_exists(&self.config, nickname) {
            diesel::update(
                userTable.filter(
                    server_id.eq(&self.config.table_id)
                ).filter(
                    user_nickname.eq(nickname)
                )

            ).set(is_banned.eq(set_banned)).execute(
                &self.config.connection

            ).ok();
            true

        } else {
            false
        }
    }

}

