// External Dependencies ------------------------------------------------------
use chrono;


// Voice Greeting Abstraction -------------------------------------------------
pub struct Greeting {
    pub nickname: String,
    pub effect: String,
    pub last_played: i64,
    pub permanent: bool
}

impl Greeting {
    pub fn new(nickname: String, effect: String, permanent: bool) -> Greeting {
        Greeting {
            nickname: nickname,
            effect: effect,
            last_played: chrono::Local::now().num_seconds_from_unix_epoch(),
            permanent: permanent
        }
    }
}

