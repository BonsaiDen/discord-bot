// External Dependencies ------------------------------------------------------
use diesel;
use diesel::prelude::*;


// Internal Dependencies ------------------------------------------------------
use super::Server;
use ::db::models::{User, NewUser};
use ::db::schema::users::dsl::{server_id, nickname as user_nickname, is_banned};
use ::db::schema::users::table as userTable;


// Server Ban Interface -------------------------------------------------------
impl Server {

    pub fn list_bans(&self) -> Vec<User> {
        userTable.filter(server_id.eq(&self.table_id))
                 .filter(is_banned.eq(true))
                 .order(user_nickname)
                 .load::<User>(&self.connection)
                 .unwrap_or_else(|_| vec![])
    }

    pub fn add_ban(&mut self, nickname: &str) -> bool {

        // TODO dry
        let q = userTable.filter(server_id.eq(&self.table_id))
                         .filter(user_nickname.eq(nickname));

        // Create user
        if q.count().get_result(&self.connection).unwrap_or(0) == 0 {
            diesel::insert(&NewUser {
                        server_id: &self.table_id,
                        nickname: nickname,
                        is_admin: false,
                        is_uploader: false,
                        is_banned: false

                    }).into(userTable)
                   .execute(&self.connection)
                   .expect("add_alias failed to insert into database");

        }

        // Ban user
        let q = userTable.filter(server_id.eq(&self.table_id))
                         .filter(user_nickname.eq(nickname))
                         .filter(is_banned.eq(false));

        if q.count().get_result(&self.connection).unwrap_or(0) > 0 {
            diesel::update(q)
                   .set(is_banned.eq(true))
                   .execute(&self.connection)
                   .expect("add_ban failed to update database");
            true

        } else {
            false
        }

    }

    pub fn remove_ban(&mut self, nickname: &str) -> bool {
        let q = userTable.filter(server_id.eq(&self.table_id))
                         .filter(user_nickname.eq(nickname))
                         .filter(is_banned.eq(true));

        if q.count().get_result(&self.connection).unwrap_or(0) > 0 {
            diesel::update(q)
                   .set(is_banned.eq(false))
                   .execute(&self.connection)
                   .expect("remove_ban failed to update database");
            true

        } else {
            false
        }

    }

}

