// Discord Dependencies -------------------------------------------------------
use discord::model::ChannelId;


// Internal Dependencies ------------------------------------------------------
use ::core::{Channel, Member};
use super::Server;


// Server Channel Interface ---------------------------------------------------
impl Server {

    pub fn has_channel(&self, channel_id: &ChannelId) -> bool {
        self.channels.contains_key(channel_id)
    }

    pub fn add_channel(&mut self, channel: Channel) {
        let channel_id = channel.id;
        self.channels.insert(channel_id, channel);
        info!("{} {} added", self, self.channels[&channel_id]);
    }

    pub fn update_channel(&mut self, channel: Channel) {
        let channel_id = channel.id;
        if self.channels.contains_key(&channel_id) {

            let bitrate = if let Some(server_channel) = self.channels.get_mut(&channel.id) {
                server_channel.update(channel);
                Some(server_channel.bitrate())

            } else {
                None
            };

            if let Some(bitrate) = bitrate {
                self.update_effect_bitrate(bitrate);
            }

            info!("[{}] {} updated", self, self.channels[&channel_id]);
        }
    }

    pub fn remove_channel(&mut self, channel: Channel) {
        self.channels.remove(&channel.id);
        info!("{} {} removed", self, channel);
    }

    pub fn channel_name(&self, channel_id: &ChannelId) -> Option<String> {
        self.channels.get(channel_id).map(|channel| channel.name.to_string())
    }

    pub fn get_channel_id(&self, channel_name: &str) -> Option<ChannelId> {
        for channel in self.channels.values() {
            if channel.name == channel_name {
                return Some(channel.id);
            }
        }
        None
    }

    pub fn channel_voice_members(&self, channel_id: &ChannelId) -> Vec<&Member> {
        self.channels.get(channel_id).map(|channel| {
            channel.voice_users().iter().filter_map(|user_id| {
                self.members.get(user_id)

            }).collect()

        }).unwrap_or_else(Vec::new)
    }

}

