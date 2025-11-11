use serde::{Serialize, Deserialize};
use sqlx::FromRow;

enum Outcome {
    Yes,
    No
}

#[derive(Deserialize, Serialize, FromRow, Debug)]
pub struct Market {
    id: u32,
    name: String,
    desc: String,
    img_url: String,
    volume: u32,
    end_date: String,
    creater_admin_id: u32,
    outcome: Outcome
}