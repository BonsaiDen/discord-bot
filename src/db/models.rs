// Internal Dependencies ------------------------------------------------------
use super::schema::*;


// Model Abstractions ---------------------------------------------------------
#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub server_id: String,
    pub nickname: String,
    pub is_admin: bool,
    pub is_uploader: bool,
    pub is_banned: bool
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub server_id: &'a str,
    pub nickname: &'a str,
    pub is_admin: bool,
    pub is_uploader: bool,
    pub is_banned: bool
}

#[derive(Queryable)]
pub struct Greeting {
    pub id: i32,
    pub server_id: String,
    pub nickname: String,
    pub effect_name: String
}

#[derive(Insertable)]
#[table_name="greetings"]
pub struct NewGreeting<'a> {
    pub server_id: &'a str,
    pub nickname: &'a str,
    pub effect_name: &'a str
}

#[derive(Queryable)]
pub struct Alias {
    pub id: i32,
    pub server_id: String,
    pub name: String,
    pub effect_names: String
}

#[derive(Insertable)]
#[table_name="aliases"]
pub struct NewAlias<'a> {
    pub server_id: &'a str,
    pub name: &'a str,
    pub effect_names: &'a str
}

#[derive(Queryable)]
pub struct Effect {
    pub id: i32,
    pub server_id: String,
    pub name: String,
    pub uploader: String,
    pub peak_db: f32,
    pub duration_ms: i32,
    pub silent_start_samples: i32,
    pub silent_end_samples: i32,
    pub transcript: String
}

#[derive(Insertable)]
#[table_name="effects"]
pub struct NewEffect<'a> {
    pub server_id: &'a str,
    pub name: &'a str,
    pub uploader: &'a str,
    pub peak_db: f32,
    pub duration_ms: i32,
    pub silent_start_samples: i32,
    pub silent_end_samples: i32,
    pub transcript: &'a str
}

#[derive(Queryable)]
pub struct Streamer {
    pub id: i32,
    pub server_id: String,
    pub channel_id: String,
    pub twitch_nick: String,
    pub is_online: bool
}

#[derive(Insertable)]
#[table_name="streamers"]
pub struct NewStreamer<'a> {
    pub server_id: &'a str,
    pub channel_id: String,
    pub twitch_nick: &'a str,
    pub is_online: bool
}

