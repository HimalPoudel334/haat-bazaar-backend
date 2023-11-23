use crate::config::ApplicationConfiguration;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::r2d2::{PoolError, PooledConnection};
use diesel::sqlite::SqliteConnection;

pub type SqliteConnectionPool = Pool<ConnectionManager<SqliteConnection>>;
pub type PooledSqliteConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

pub fn establish_connection(app_config: &ApplicationConfiguration) -> SqliteConnectionPool {
    let database_url = app_config.database_url.to_owned();

    let conn_manager = ConnectionManager::<SqliteConnection>::new(database_url);
    match Pool::builder().build(conn_manager) {
        Ok(pool) => pool,
        Err(err) => {
            println!("Error while creating database connection pool: {}", err);
            std::process::exit(1);
        }
    }
}
