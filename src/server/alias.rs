// Internal Dependencies ------------------------------------------------------
use super::Server;


// Server Aliases Interface ---------------------------------------------------
impl Server {

    pub fn has_alias(&self, alias_name: &str) -> bool {
        self.config.aliases.contains_key(alias_name)
    }

    #[allow(ptr_arg)]
    pub fn add_alias(&mut self, alias_name: &str, effect_names: &Vec<String>) {
        self.config.aliases.insert(alias_name.to_string(), effect_names.clone());
        self.store_config().expect("add_alias failed to store config.");
    }

    pub fn remove_alias(&mut self, alias_name: &str) {
        self.config.aliases.remove(alias_name);
        self.store_config().expect("remove_alias failed to store config.");
    }

    pub fn list_aliases(&self) -> Vec<(&String, &Vec<String>)> {
        self.config.aliases.iter().map(|(name, effects)| {
            (name, effects)

        }).collect()
    }

}

