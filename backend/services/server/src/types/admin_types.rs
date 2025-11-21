use serde::{Serialize, Deserialize};
use validator::Validate;


#[derive(Serialize, Deserialize, Debug)]
pub struct Admin {
    pub id: u64,
    pub email: String,
    pub name: String,
    pub password: String,
}

