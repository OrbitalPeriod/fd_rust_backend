#![allow(unused_imports, dead_code)]

use actix_web::{
    web::{self, route, Data},
    App, HttpServer,
};
use sqlx::{Pool, Postgres};
use tracing::{info, warn};
use tracing_actix_web::TracingLogger;

mod handlers;
mod models;
mod routes;
mod utils;

#[actix_web::main]
async fn main() {
    set_logging();
    info!("Starting server");

    

    dotenv::dotenv().expect("Failed to read .env file");
    let pool = configure_sql_connection().await;
    info!("Connected to database");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .wrap(TracingLogger::default())
            .configure(routes::config)
    })
    .bind(("127.0.0.1", 8080))
    .expect("Failed to bind to address")
    .run()
    .await
    .expect("error running server");
}

fn set_logging() {
    let level = if cfg!(debug_assertions) {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt().with_max_level(level).init();
}

async fn configure_sql_connection() -> Pool<Postgres> {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    sqlx::postgres::PgPoolOptions::new()
        .connect(&url)
        .await
        .expect("Something went wrong connecting to the db")
}

//todo: implement serialzize for Position enum

//cargo watch -x 'run' -c
