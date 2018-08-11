use diesel;
use lib_error::*;
use {embedded_migrations, Connection, SqlProvider};

// In memory Sqlite store used during testing.
pub struct SqlStoreInMemory;

impl SqlProvider for SqlStoreInMemory {
    fn connection(&self) -> Result<Connection> {
        use diesel::sqlite::SqliteConnection;
        use diesel::Connection as DieselConnection;

        let connection = SqliteConnection::establish(":memory:").expect("in memory database");
        embedded_migrations::run(&connection).expect("create tables");
        Ok(connection)
    }
}
