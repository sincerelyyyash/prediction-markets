use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct SignUpUserInput {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 2, message = "Name must be atleast 2 characters"))]
    pub name: String,

    #[validate(length(min = 8, message = "Password must be atleast 8 characters"))]
    pub password: String,
}

#[derive(Deserialize, Validate, Debug)]
pub struct LoginUserInput {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be atleast 8 characters long"))]
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct SplitSchema {
    pub id: u64,
    pub market_id: u64,
    pub amount: u64,
}

#[derive(Deserialize, Debug)]
pub struct MergeSchema {
    pub id: u64,
    pub market_id: u64,
}
