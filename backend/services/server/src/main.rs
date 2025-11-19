mod controllers;
mod types;
mod utils;
mod models;

use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use sqlx::postgres::PgPoolOptions;
use std::env;
use dotenvy::dotenv;
use crate::controllers::user_controller::{signup_user, signin_user};
use crate::controllers::admin_controller::{signin_admin};

async fn health()-> impl Responder {
    HttpResponse::Ok()
    .content_type("application/json")
    .body(r#"{"status": "Ok"}"#)
}

async fn run()-> std::io::Result<()> {

    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set in .env");

    let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await
    .expect("Failed to create Postgres pool");

    println!("Connected to Postgres Database");


    HttpServer::new(move|| App::new()
    .app_data(web::Data::new(pool.clone()))
    .service(signup_user)
    .service(signin_user)
    .route("/health", web::get().to(health)))
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

fn main()-> std::io::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");
    runtime.block_on(run())
}