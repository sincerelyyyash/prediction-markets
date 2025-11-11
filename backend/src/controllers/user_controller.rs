use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use bcrypt::{hash, DEFAULT_COST};
use serde_json::json;
use validator::Validate;
use sqlx::PgPool;
use std::env;

#[derive(Deserialize, Validate, Debug)]
struct signUpUserInput {
    #[validate(email(message = "Invalid email format"))]
    email: String,

    #[Validate(length(min=2, message= "Name must be atleast 2 characters"))]
    name: String,

    #[Validate(length(min=8, message= "Password must be atleast 8 characters"))]
    password: String,
}

#[derive(Deserialize, Validate, Debug)]
struct loginUserInput {
    #[Validate(email(message = "Invalid email format"))]
    email: String,

    #[Validate(length(min=8, message= "Password must be atleast 8 characters long"))]
    password: String,
}

#[derive(Deserialize, Validate, Debug)]
struct splitSchema {
    #[Validate(length(min= 2, message= "User id invalid"))]
    id: u32,

    #[Validate(length(min=2,message= "Market id invalid"))]
    marketId: u32,

    #[Validate(length(min=1, message="amount cannot be less than 1"))]
    amount: u64,
}

#[derive(Deserialize, Validate, Debug)]
struct mergeSchema {
    #[Validate(length(min=2, message ="Invalid user Id"))]
    id: u32,

    #[Validate(length(min=2,message= "Market id invalid"))]
    marketId: u32,

}


#[post("user/signup")]
async fn signup_user(db_pool: web::Data<PgPool>, req: web::Json<signUpUserInput>) -> impl Responder {
    if let Err(e) = req.validate(){
        return HttpResponse::BadRequest().json(json!({
            "status":"error",
            "message": e.to_string()
        }));
    }

    let hashed_password = match hash(&req.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status":"error",
                "message":"Failed to sign up user"
            }))
        }
    };

    let user = sqlx::query!(
        r#"
        INSERT INTO users(email, name, hashed_password)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        req.email,
        req.name,
        req.password
    )
    .fetch_one(db_pool.get_ref())
    .await;

    match user {
        Ok(record) => HttpResponse::Ok().json(json!({
            "status":"success",
            "message":"User registered Succesfully",
            "user_id": record.id 
        })),
        Err(e) => {
            eprintln!("Database error: {:?}",e );
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "could not save user"
            }))
        }
    }
}

#[post("user/signin")]
async fn signin_user(db_pool: web::Data<PgPool>, req: web::Json<loginUserInput>) -> impl Responder {
    if let Err(e) = req.validate(){
        return HttpResponse::BadRequest().json(json!({
            "status":"error",
            "message": e.to_string()
        }))
    }


    let result = sqlx::query_as!(
        User,
        r#"
        SELECT id, name, email, password, balance FROM users WHERE email =$1
        "#
        req.email
    )
    .fetch_optional(pool.get_ref())
    .await;

    let existing_user = match result {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(json!({
                "status":"error",
                "message": "Invalid email or User does not exists"
            }));
        }
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Database query failed"
            }));
        }
    };

    let hashed_password = match hash(&req.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status":"error",
                "message":"Failed to sign up user"
            }))
        }
    };

    let is_valid = verify(&hashed_password, &existing_user.password).unwrap_or(false);
    if !is_valid {
        return HttpResponse::Unauthorized().json(json!({
            "status":"error",
            "message": "Incorrect password"
        }));
    }

    let jwt_token =env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = match create_jwt(&existing_user.id, &jwt_token){
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status":"error",
                "message":"Failed to sign in user"
            }))
        }

    };

    HttpResponse::Ok().json(json!({
        "status":"success",
        "message":"Sign in successfully",
        "token": token,
        "user":{
            "id": existing_user.id,
            "email": existing_user.email,
            "name" : existing_user.name,
            "balance": existing_user.balance
        }
    }))
}
