// Internal Dependencies ------------------------------------------------------
use super::Server;


// Server Ban Interface -------------------------------------------------------
impl Server {

    pub fn list_bans(&self) -> Vec<String> {
        self.config.banned.iter().map(|n| n.to_string()).collect()
    }

    #[allow(ptr_arg)]
    pub fn add_ban(&mut self, nickname: &String) -> bool {
        if !self.config.banned.contains(nickname) {
            self.config.banned.push(nickname.to_string());
            self.store_config().expect("add_ban failed to store config.");
            true

        } else {
            false
        }
    }

    #[allow(ptr_arg)]
    pub fn remove_ban(&mut self, nickname: &String) -> bool {
        if self.config.banned.contains(nickname) {
            self.config.banned.retain(|n| n != nickname);
            self.store_config().expect("remove_ban failed to store config.");
            true

        } else {
            false
        }
    }

}

