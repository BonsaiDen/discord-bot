// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::effects::Effect;
use ::actions::SendMessage;
use ::bot::{Bot, BotConfig};
use ::core::message::Message;
use ::core::event::EventQueue;
use ::text_util::list_words;
use ::actions::{Action, ActionGroup};


// Action Implementation ------------------------------------------------------
pub struct ListEffects {
    message: Message,
    patterns: Option<Vec<String>>
}

impl ListEffects {

    pub fn all(message: Message) -> Box<ListEffects> {
        Box::new(ListEffects {
            message: message,
            patterns: None
        })
    }

    pub fn matching(message: Message, patterns: Vec<String>) -> Box<ListEffects> {
        Box::new(ListEffects {
            message: message,
            patterns: Some(patterns)
        })
    }

}

impl Action for ListEffects {
    fn run(&self, bot: &mut Bot, config: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {

            if let Some(ref patterns) = self.patterns {

                let title = format!(
                    "Sound Effect matching \"{}\"", patterns.join("\", \"")
                );

                info!("{} {}", self, title);

                let effects = server.map_effects(&patterns[..], true, config);
                list_effects(&self.message, title.as_str(), effects)

            } else {
                info!("{} Listing all sound effects...", server);

                let patterns = vec![String::from("*")];
                let effects = server.map_effects(&patterns[..], true, config);
                list_effects(&self.message, "Sound Effects", effects)
            }

        } else {
            vec![]
        }

    }
}

impl fmt::Display for ListEffects {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [ListEffects]")
    }
}


// Helpers --------------------------------------------------------------------
fn list_effects(
    message: &Message,
    title: &str,
    effects: Vec<Effect>

) -> ActionGroup {

    let mut effects_names: Vec<&str> = effects.iter().map(|effect| {
        effect.name.as_str()

    }).collect();

    effects_names.sort();

    list_words(title, effects_names, 100, 4).into_iter().map(|text| {
        SendMessage::private(message, text) as Box<Action>

    }).collect()

}

