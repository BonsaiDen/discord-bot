// Discord Dependencies -------------------------------------------------------
use discord::model::ChannelId;


// Internal Dependencies ------------------------------------------------------
use ::core::Channel;
use super::Server;


// Server Channel Interface ---------------------------------------------------
impl Server {

    pub fn has_channel(&self, channel_id: &ChannelId) -> bool {
        self.channels.contains_key(channel_id)
    }

    pub fn add_channel(&mut self, channel: Channel) {
        let channel_id = channel.id;
        self.channels.insert(channel_id, channel);
        info!("{} {} added", self, self.channels.get(&channel_id).unwrap());
    }

    pub fn update_channel(&mut self, channel: Channel) {
        let channel_id = channel.id;
        if self.channels.contains_key(&channel_id) {
            if let Some(server_channel) = self.channels.get_mut(&channel.id) {
                server_channel.update(channel);
            }
            info!("[{}] {} updated", self, self.channels.get(&channel_id).unwrap());
        }
    }

    pub fn remove_channel(&mut self, channel: Channel) {
        self.channels.remove(&channel.id);
        info!("{} {} removed", self, channel);
    }

}

