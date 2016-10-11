// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::bot::{Bot, BotConfig};
use ::core::event::EventQueue;


// Modules --------------------------------------------------------------------
pub mod add_ban;
pub mod list_bans;
pub mod remove_ban;
pub mod send_message;
pub mod list_effects;
pub mod list_aliases;
pub mod play_effects;
pub mod rename_effect;
pub mod delete_effect;
pub mod list_greetings;
pub mod delete_message;
pub mod download_flac_file;
pub mod leave_server_voice;
pub mod silence_active_effects;
pub mod reload_server_configuration;


// Re-Exports -----------------------------------------------------------------
pub use self::add_ban::AddBan;
pub use self::list_bans::ListBans;
pub use self::remove_ban::RemoveBan;
pub use self::play_effects::PlayEffects;
pub use self::list_aliases::ListAliases;
pub use self::rename_effect::RenameEffect;
pub use self::delete_effect::DeleteEffect;
pub use self::delete_message::DeleteMessage;
pub use self::list_greetings::ListGreetings;
pub use self::download_flac_file::DownloadFlacFile;
pub use self::leave_server_voice::LeaveServerVoice;
pub use self::silence_active_effects::SilenceActiveEffects;
pub use self::list_effects::{ListAllEffects, ListPatternEffects};
pub use self::send_message::{SendPrivateMessage, SendPublicMessage};
pub use self::reload_server_configuration::ReloadServerConfiguration;


// General Action Abstraction -------------------------------------------------
pub type ActionGroup = Vec<Box<Action>>;

pub trait Action: fmt::Display {
    fn run(&self, &mut Bot, &BotConfig, &mut EventQueue) -> ActionGroup;
}

