// External Dependencies ------------------------------------------------------
use diesel;
use diesel::prelude::*;


// Internal Dependencies ------------------------------------------------------
use super::Server;
use ::db::models::{User, NewUser};
use ::db::schema::users::dsl::{server_id, nickname as user_nickname, is_uploader};
use ::db::schema::users::table as userTable;


// Server Uploader Interface --------------------------------------------------
impl Server {

    pub fn list_uploaders(&self) -> Vec<User> {
        userTable.filter(
            server_id.eq(&self.config.table_id)

        ).filter(is_uploader.eq(true))
         .order(user_nickname)
         .load::<User>(&self.config.connection)
         .unwrap_or_else(|_| vec![])
    }

    pub fn add_uploader(&mut self, nickname: &str) -> bool {

        // TODO dry
        let q = userTable.filter(server_id.eq(&self.config.table_id))
                         .filter(user_nickname.eq(nickname));

        // Create user
        if q.count().get_result(&self.config.connection).unwrap_or(0) == 0 {
            diesel::insert(&NewUser {
                        server_id: &self.config.table_id,
                        nickname: nickname,
                        is_admin: false,
                        is_uploader: false,
                        is_banned: false

                    }).into(userTable)
                   .execute(&self.config.connection)
                   .expect("add_alias failed to insert into database");

        }

        self.update_upload_user(nickname, true)

    }

    pub fn remove_uploader(&mut self, nickname: &str) -> bool {
        self.update_upload_user(nickname, false)
    }

    // TODO make generic and merge with ban
    fn update_upload_user(&self, nickname: &str, set_uploader: bool) -> bool {
        let q = userTable.filter(server_id.eq(&self.config.table_id))
                         .filter(user_nickname.eq(nickname))
                         .filter(is_uploader.eq(!set_uploader));

        if q.count().get_result(&self.config.connection).unwrap_or(0) > 0 {
            diesel::update(q)
                   .set(is_uploader.eq(set_uploader))
                   .execute(&self.config.connection).ok();
            true

        } else {
            false
        }
    }

}

