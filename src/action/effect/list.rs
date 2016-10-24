// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::effect::Effect;
use ::bot::{Bot, BotConfig};
use ::text_util::list_words;
use ::core::{EventQueue, Message};
use ::action::{Action, ActionGroup, MessageActions};


// Action Implementation ------------------------------------------------------
pub struct ActionImpl {
    message: Message,
    patterns: Option<Vec<String>>
}

impl ActionImpl {

    pub fn all(message: Message) -> Box<ActionImpl> {
        Box::new(ActionImpl {
            message: message,
            patterns: None
        })
    }

    pub fn matching(message: Message, patterns: Vec<String>) -> Box<ActionImpl> {
        Box::new(ActionImpl {
            message: message,
            patterns: Some(patterns)
        })
    }

}

impl Action for ActionImpl {
    fn run(&mut self, bot: &mut Bot, config: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {

            if let Some(ref patterns) = self.patterns {
                let title = format!(
                    "Sound Effect matching \"{}\"",
                    patterns.join("\", \"")
                );

                let effects = server.map_effects(&patterns[..], true, config);
                list_effects(&self.message, title.as_str(), effects)

            } else {
                let patterns = vec![String::from("*")];
                let effects = server.map_effects(&patterns[..], true, config);
                list_effects(&self.message, "Sound Effects", effects)
            }

        } else {
            vec![]
        }

    }
}

impl fmt::Display for ActionImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [ListEffects]")
    }
}


// Helpers --------------------------------------------------------------------
fn list_effects(
    message: &Message,
    title: &str,
    effects: Vec<&Effect>

) -> ActionGroup {

    let mut effects_names: Vec<&str> = effects.iter().map(|effect| {
        effect.name.as_str()

    }).collect();

    effects_names.sort();

    list_words(title, effects_names, 100, 4).into_iter().map(|text| {
        MessageActions::Send::single_private(message, text) as Box<Action>

    }).collect()

}

