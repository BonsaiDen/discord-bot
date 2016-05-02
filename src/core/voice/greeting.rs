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
            last_played: 0,
            permanent: permanent
        }
    }
}

