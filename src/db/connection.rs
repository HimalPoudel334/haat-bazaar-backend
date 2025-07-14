use diesel::r2d2::{ConnectionManager, Pool};
use diesel::r2d2::{PoolError, PooledConnection};
use diesel::sqlite::SqliteConnection;

pub type SqliteConnectionPool = Pool<ConnectionManager<SqliteConnection>>;
pub type PooledSqliteConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

pub fn establish_connection(database_url: &String) -> SqliteConnectionPool {
    let conn_manager = ConnectionManager::<SqliteConnection>::new(database_url);
    match Pool::builder().build(conn_manager) {
        Ok(pool) => pool,
        Err(err) => {
            println!("Error while creating database connection pool: {}", err);
            std::process::exit(1);
        }
    }
}

pub fn get_conn(pool: &SqliteConnectionPool) -> PooledSqliteConnection {
    pool.get().unwrap()
}

pub fn get_db_connection_from_pool(
    pool: &SqliteConnectionPool,
) -> Result<PooledSqliteConnection, PoolError> {
    let result = pool.get().unwrap();
    Ok(result)
}
