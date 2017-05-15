// External Dependencies ------------------------------------------------------
use diesel;
use diesel::prelude::*;


// Modules --------------------------------------------------------------------
pub mod models;
pub mod schema;


// Internal Dependencies ------------------------------------------------------
use ::server::ServerConfig;
use self::models::{User, NewUser};
use self::schema::users::dsl::{server_id, nickname as user_nickname};
use self::schema::users::table as userTable;


// Database Abstractions ------------------------------------------------------
pub fn user_exists(
    config: &ServerConfig,
    nickname: &str

) -> bool {
    userTable.filter(
        server_id.eq(&config.table_id)

    ).filter(
        user_nickname.eq(nickname)

    ).count().get_result(&config.connection).unwrap_or(0) > 0
}

pub fn get_user_or_default(
    config: &ServerConfig,
    nickname: &str

) -> User {
    userTable.filter(
        server_id.eq(&config.table_id)

    ).filter(
        user_nickname.eq(nickname)

    ).first::<User>(&config.connection).ok().unwrap_or_else(|| {
         User {
             id: -1,
             server_id: config.table_id.clone(),
             nickname: nickname.to_string(),
             is_admin: false,
             is_uploader: false,
             is_banned: false
         }
     })
}

pub fn create_user_if_not_exists(
    config: &ServerConfig,
    nickname: &str

) -> Result<(), String> {
    if userTable.filter(
        server_id.eq(&config.table_id)

    ).filter(
        user_nickname.eq(nickname)

    ).count().get_result(&config.connection).unwrap_or(0) == 0 {
        diesel::insert(&NewUser {
            server_id: &config.table_id,
            nickname: nickname,
            is_admin: false,
            is_uploader: false,
            is_banned: false

        }).into(userTable).execute(&config.connection).and_then(|_| {
            Ok(())

        }).map_err(|err| {
            err.to_string()
        })

    } else {
        Ok(())
    }
}

