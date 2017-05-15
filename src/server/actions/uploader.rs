// External Dependencies ------------------------------------------------------
use diesel;
use diesel::prelude::*;


// Internal Dependencies ------------------------------------------------------
use super::super::Server;
use ::db::models::User;
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
        ::db::create_user_if_not_exists(&self.config, nickname).ok();
        self.update_upload_user(nickname, true)
    }

    pub fn remove_uploader(&mut self, nickname: &str) -> bool {
        self.update_upload_user(nickname, false)
    }

    fn update_upload_user(&self, nickname: &str, set_uploader: bool) -> bool {
        if ::db::user_exists(&self.config, nickname) {
            diesel::update(
                userTable.filter(
                    server_id.eq(&self.config.table_id)
                ).filter(
                    user_nickname.eq(nickname)
                )

            ).set(is_uploader.eq(set_uploader)).execute(
                &self.config.connection

            ).ok();
            true

        } else {
            false
        }
    }

}

