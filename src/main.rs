use actix_web::{web::Data, App, HttpServer};
use dotenvy::dotenv;

use crate::db::connection;

mod base_types;
mod config;
mod contracts;
mod db;
mod handlers;
mod models;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //load the environenment variables
    dotenv().ok();

    //create the app config struct
    let app_config = config::ApplicationConfiguration::init();

    //setting up the sqlite database
    let sqlitedb_pool: connection::SqliteConnectionPool =
        connection::establish_connection(&app_config);

    println!("Server started on http://localhost:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_config.clone()))
            .app_data(Data::new(sqlitedb_pool.clone()))
            .configure(routes::app_routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
