use std::{error::Error, sync::Arc};

use actix_files as fs;
use actix_web::{web::Data, App, HttpServer};
use dotenvy::dotenv;
use middlewares::jwt_middleware;
use reqwest::Client;

use crate::{
    db::connection,
    services::{
        fcm_notification_service::FcmNotificationServiceImpl,
        notification_service::NotificationService,
    },
};

mod base_types;
mod config;
mod contracts;
mod db;
mod handlers;
mod middlewares;
mod models;
mod routes;
mod schema;
mod services;
mod utils;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let app_config = config::ApplicationConfiguration::init();
    let email_config = config::EmailConfiguration::init();
    let company_config = config::EmailConfiguration::init();

    let db_pool: connection::SqliteConnectionPool =
        connection::establish_connection(&app_config.database_url);

    let client: Client = Client::new();

    let service_account_key_path = &app_config.firebase_service_account_key_path;
    let fcm_client = utils::fcm_client::FcmClient::new(service_account_key_path).await?;
    let fcm_client_arc = Arc::new(fcm_client);

    let notification_service_impl =
        FcmNotificationServiceImpl::new(fcm_client_arc.clone(), db_pool.clone());

    let notification_service: Arc<dyn NotificationService> = Arc::new(notification_service_impl);

    std::fs::create_dir_all(&app_config.product_extraimages_path)?;
    std::fs::create_dir_all(&app_config.product_thumbnail_path)?;

    let server_address = app_config.server_address.clone();
    let server_port = app_config.server_port.clone();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_config.clone()))
            .app_data(Data::new(company_config.clone()))
            .app_data(Data::new(email_config.clone()))
            .app_data(Data::new(db_pool.clone()))
            .app_data(Data::new(client.clone()))
            .app_data(Data::new(notification_service.clone()))
            .wrap(jwt_middleware::JwtAuthentication)
            .configure(routes::app_routes)
            .service(fs::Files::new("/images", "./images"))
    })
    .bind((server_address, server_port))
    .map_err(|e| Box::new(e) as Box<dyn Error>)?
    .run()
    .await?;

    println!("Server started on http://localhost:8080");

    Ok(())
}
