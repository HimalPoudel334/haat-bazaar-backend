use std::{error::Error, sync::Arc};

use actix_files as fs;
use actix_web::{web::Data, App, HttpServer};
use dotenvy::dotenv;
use middlewares::jwt_middleware;
use reqwest::Client;

use crate::db::connection;

mod base_types;
mod config;
mod contracts;
mod db;
mod handlers;
mod middlewares;
mod models;
mod routes;
mod schema;
mod utils;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //load the environenment variables
    dotenv().ok();

    //create the app config struct
    let app_config = config::ApplicationConfiguration::init();

    //setting up the sqlite database
    let db_pool: connection::SqliteConnectionPool =
        connection::establish_connection(&app_config.database_url);

    //setting up the request client
    let client: Client = Client::new();

    // --- FCM Client Setup ---
    let service_account_key_path = &app_config.firebase_service_account_key_path;
    let fcm_client = utils::fcm_client::FcmClient::new(service_account_key_path).await?;
    let fcm_client_arc = Arc::new(fcm_client);

    //create a directory for uploading images
    std::fs::create_dir_all(&app_config.product_extraimages_path)?;
    std::fs::create_dir_all(&app_config.product_thumbnail_path)?;

    let server_address = app_config.server_address.clone();
    let server_port = app_config.server_port.clone();

    println!("Server started on http://localhost:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_config.clone()))
            .app_data(Data::new(db_pool.clone()))
            .app_data(Data::new(client.clone()))
            .app_data(Data::new(fcm_client_arc.clone()))
            .wrap(jwt_middleware::JwtAuthentication)
            .configure(routes::app_routes)
            .service(fs::Files::new("/images", "./images"))
    })
    .bind((server_address, server_port))
    .map_err(|e| Box::new(e) as Box<dyn Error>)?
    .run()
    .await?;

    Ok(())
}
