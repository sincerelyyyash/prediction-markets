use serde::{Serialize, Deserialize};
use validator::Validate;


#[derive(Serialize, Deserialize, Debug)]
pub struct Admin {
    pub id: u64,
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct CreateEventAdminRequest {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub category: String,
    pub resolved_at: Option<String>,
    pub created_by: u64,
    pub outcomes: Vec<CreateOutcomeInput>
}

#[derive(Serialize, Deserialize, Debug)]

pub struct CreateOutcomeInput {
    pub name: String,
    pub status: String,
}