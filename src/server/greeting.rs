// Discord Dependencies -------------------------------------------------------
use discord::model::UserId;


// Internal Dependencies ------------------------------------------------------
use ::bot::BotConfig;
use ::effect::Effect;
use super::Server;


// Server Greeting Interface --------------------------------------------------
impl Server {

    pub fn get_greeting(
        &self,
        member_id: &UserId,
        bot_config: &BotConfig

    ) -> Option<Vec<&Effect>> {
        if let Some(member) = self.members.get(member_id) {

            // User specific greeting
            if let Some(effect_name) = self.config.greetings.get(&member.nickname) {
                Some(self.map_effects(
                    &[effect_name.to_string()],
                    false,
                    bot_config
                ))

            // Default greeting
            } else if let Some(effect_name) = self.config.greetings.get("default") {
                Some(self.map_effects(
                    &[effect_name.to_string()],
                    false,
                    bot_config
                ))

            } else {
                None
            }

        } else {
            None
        }
    }

    pub fn has_greeting(&self, nickname: &str) -> bool {
        self.config.greetings.contains_key(nickname)
    }

    pub fn add_greeting(&mut self, nickname: &str, effect_name: &str) {
        self.config.greetings.insert(nickname.to_string(), effect_name.to_string());
        self.store_config().expect("add_greeting failed to store config.");
    }

    pub fn remove_greeting(&mut self, nickname: &str) {
        self.config.greetings.remove(nickname);
        self.store_config().expect("remove_greeting failed to store config.");
    }

    pub fn list_greetings(&self) -> Vec<(&String, &String)> {
        self.config.greetings.iter().map(|(nickname, effect)| {
            (nickname, effect)

        }).collect()
    }

}

