use actix_files as fs;
use actix_web::{web::Data, App, HttpServer};
use dotenvy::dotenv;
use reqwest::Client;

use crate::db::connection;

mod base_types;
mod config;
mod contracts;
mod db;
mod handlers;
mod models;
mod routes;
mod schema;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //load the environenment variables
    dotenv().ok();

    //create the app config struct
    let app_config = config::ApplicationConfiguration::init();

    //setting up the sqlite database
    let db_pool: connection::SqliteConnectionPool =
        connection::establish_connection(&app_config.database_url);

    //setting up the request client
    let client: Client = Client::new();

    //create a directory for uploading images
    std::fs::create_dir_all(&app_config.product_extraimages_path)?;
    std::fs::create_dir_all(&app_config.product_thumbnail_path)?;

    println!("Server started on http://localhost:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_config.clone()))
            .app_data(Data::new(db_pool.clone()))
            .app_data(Data::new(client.clone()))
            .configure(routes::app_routes)
            .service(fs::Files::new("/images", "./images"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
