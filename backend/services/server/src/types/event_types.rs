use serde::{Serialize, Deserialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct CreateEventRequest {
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

#[derive (Serialize, Deserialize, Validate, Debug)]
pub struct ResolveEventRequest {
    pub event_id: u64,
    pub status: String,
    pub resolved_at: Option<String>,
    pub winning_outcome_id: u64,
}

#[derive (Serialize, Deserialize, Validate, Debug)]
pub struct UpdateEventRequest {
    pub event_id: u64,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub category: Option<String>,
}

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct DeleteEventRequest {
    pub event_id: u64,
}