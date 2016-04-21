// Discord Dependencies -------------------------------------------------------
use discord::{ChannelRef};
use discord::model::{ChannelId, MessageId, ServerId};


// Internal Dependencies ------------------------------------------------------
use super::{command, Handle, Server, User};


// Message Abstraction --------------------------------------------------------
pub struct Message<'a> {
    pub id: &'a MessageId,
    pub server_id: &'a ServerId,
    pub channel_id: &'a ChannelId,
    pub author: &'a User,
    pub content: &'a str,
    pub was_edited: bool
}


// Message Handling -----------------------------------------------------------
impl<'a> Message<'a> {

    pub fn handle(
        &self,
        handle: &mut Handle,
        server: &mut Server,
        unique_server: bool
    ) {

        if self.author.id == handle.user_id() {
            debug!("[Message] Ignored response message from bot.");
            return;

        } else {
            self.log(handle, server);
        }

        if self.content.starts_with("!") {

            let mut split = self.content.split(" ");
            let name = split.next().unwrap_or("!");
            let command = command::from_args(
                &name[1..],
                split.collect(),
                unique_server
            );

            if let Some(responses) = command.execute(handle, server, self.author) {
                for response in responses {
                    handle.send_message_to_user(&self.author.id, &response);
                }
            }

        }

    }

    fn log(&self, handle: &mut Handle, server: &mut Server) {
        match handle.find_channel_by_id(&self.channel_id) {

            Some(ChannelRef::Public(_, channel)) => {
                info!(
                    "[Message] [{}#{}] {}: {}",
                    server, channel.name,
                    self.author.nickname,
                    self.content
                );
            }

            Some(ChannelRef::Private(channel)) => {

                if self.author.name == channel.recipient.name {
                    info!(
                        "[Message] [{}] [Private] {}: {}",
                        server,
                        self.author.nickname,
                        self.content
                    );

                } else {
                    info!(
                        "[Message] [{}] [Private] To {}#{}: {}",
                        server,
                        channel.recipient.name, channel.recipient.discriminator,
                        self.content
                    );
                }

            }

            None => info!(
                "[Message] [{}] [Unknown Channel] {}: {}",
                server,
                self.author.nickname,
                self.content
            )
        }
    }

}

