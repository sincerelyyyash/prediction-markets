use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub struct Admin {
    pub id: u64,
    pub email: String,
    pub name: String,
    pub password: String,
}
