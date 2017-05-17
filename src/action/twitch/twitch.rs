// STD Dependencies -----------------------------------------------------------
use std::io::Read;


// External Dependencies ------------------------------------------------------
use serde_json;
use hyper::Client;
use hyper::header::{Connection, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel};


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;


// Helper Structures ----------------------------------------------------------
header! { (ClientId, "Client-Id") => [String] }

#[derive(Deserialize, Debug, Default)]
pub struct Links {
    #[serde(rename(deserialize = "self"))]
    url: String
}

#[derive(Deserialize, Debug)]
struct TwitchResponse {
    stream: Option<Stream>
}

#[derive(Deserialize, Debug, Default)]
pub struct Stream {
    pub game: String,
    #[serde(rename(deserialize = "_links"))]
    pub links: Links,
    pub stream_type: String,
    #[serde(rename(deserialize = "video_height"))]
    pub resolution: usize,
    pub viewers: usize
}

#[derive(Deserialize, Debug, Default)]
pub struct Channel {
    pub url: String,
    pub display_name: String
}


// Twitch Integration ---------------------------------------------------------
pub fn get_channel(config: &BotConfig, channel_name: &str) -> Result<Channel, String> {

    let client = Client::new();
    let url = format!("https://api.twitch.tv/kraken/channels/{}", channel_name);
    client.get(&url)
        .header(ContentType(Mime(TopLevel::Application, SubLevel::Ext("vnd.twitchtv.v3+json".to_string()), vec![])))
        .header(ClientId(config.twitch_client_id.clone()))
        .header(Connection::close())
        .send()
        .map_err(|err| err.to_string())
        .and_then(|mut res| {
            let mut body = String::new();
            res.read_to_string(&mut body)
               .map_err(|err| err.to_string())
               .map(|_| body)

        }).map(|response| {
            serde_json::from_str::<Channel>(&response).map_err(|err| err.to_string())

        }).and_then(|res| res)

}

pub fn get_stream(config: &BotConfig, channel_name: &str) -> Result<Stream, String> {

    let client = Client::new();
    let url = format!("https://api.twitch.tv/kraken/streams/{}", channel_name);
    client.get(&url)
        .header(ContentType(Mime(TopLevel::Application, SubLevel::Ext("vnd.twitchtv.v3+json".to_string()), vec![])))
        .header(ClientId(config.twitch_client_id.clone()))
        .header(Connection::close())
        .send()
        .map_err(|err| err.to_string())
        .and_then(|mut res| {
            let mut body = String::new();
            res.read_to_string(&mut body)
               .map_err(|err| err.to_string())
               .map(|_| body)

        }).map(|response| {
            serde_json::from_str::<TwitchResponse>(&response).map_err(|err| err.to_string())

        }).and_then(|res| {
            res.and_then(|s| {
                Ok(s.stream.unwrap_or_else(Default::default))
            })
        })

}

