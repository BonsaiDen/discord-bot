// Discord Dependencies -------------------------------------------------------
use discord::model::UserId;
use discord::model::User as DiscordUser;


// User Abstraction -----------------------------------------------------------
pub struct User {
    pub id: UserId,
    pub name: String,
    pub nickname: String
}


// User Implementation --------------------------------------------------------
impl User {

    pub fn new(user: &DiscordUser) -> User {
        User {
            id: user.id.clone(),
            name: user.name.to_string(),
            nickname: format!("{}#{}", user.name, user.discriminator)
        }
    }

}

