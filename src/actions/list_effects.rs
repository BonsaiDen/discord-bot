// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::effects::Effect;
use ::bot::{Bot, BotConfig};
use ::core::message::Message;
use ::core::event::EventQueue;
use ::actions::SendPrivateMessage;
use ::text_util::list_words;
use ::actions::{Action, ActionGroup};


// List Sound Effects Action --------------------------------------------------
pub struct ListAllEffects {
    message: Message
}

impl ListAllEffects {
    pub fn new(message: Message) -> Box<ListAllEffects> {
        Box::new(ListAllEffects {
            message: message
        })
    }
}

impl Action for ListAllEffects {
    fn run(&self, bot: &mut Bot, config: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {

            info!("{} Listing all sound effects...", server);

            let patterns = vec![String::from("*")];
            let effects = server.map_effects(&patterns[..], true, config);
            list_effects(&self.message, "Sound Effects", effects)

        } else {
            vec![]
        }

    }
}

impl fmt::Display for ListAllEffects {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [ListAllEffects]")
    }
}


// List Sound Effects Action --------------------------------------------------
pub struct ListPatternEffects {
    message: Message,
    patterns: Vec<String>
}

impl ListPatternEffects {
    pub fn new(message: Message, patterns: Vec<String>) -> Box<ListPatternEffects> {
        Box::new(ListPatternEffects {
            message: message,
            patterns: patterns
        })
    }
}

impl Action for ListPatternEffects {
    fn run(&self, bot: &mut Bot, config: &BotConfig, _: &mut EventQueue) -> ActionGroup {

        if let Some(server) = bot.get_server(&self.message.server_id) {

            let title = format!(
                "Sound Effect matching \"{}\"", self.patterns.join("\", \"")
            );

            info!("{} {}", self, title);

            let effects = server.map_effects(&self.patterns[..], true, config);
            list_effects(&self.message, title.as_str(), effects)

        } else {
            vec![]
        }

    }
}

impl fmt::Display for ListPatternEffects {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Action] [ListPatternEffects]")
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
        SendPrivateMessage::new(message, text) as Box<Action>

    }).collect()

}

