// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Discord Dependencies -------------------------------------------------------
use discord::model::ServerId;


// Server Abstraction ---------------------------------------------------------
#[derive(Debug)]
pub struct Server {
    pub id: ServerId,
    pub name: String,
    pub channel_count: usize,
    pub member_count: usize
}

impl Server {

    pub fn id(&self) -> &ServerId {
        &self.id
    }

    pub fn new(id: ServerId) -> Server {
        Server {
            id: id,
            name: "Unknown".to_string(),
            channel_count: 0,
            member_count: 0
        }
    }

}

// Traits  --------------------------------------------------------------------
impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}/{}>", self.name, self.id.0)
    }
}

