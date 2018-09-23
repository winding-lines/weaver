use lib_error::*;
use crate::{embedded_migrations, Connection, SqlProvider};

// In memory Sqlite store used during testing. The database disappears when the
// connection is closed so we pass in a function to initialize the
// database content for every connection.
pub struct SqlStoreInMemory<F> {
    initializer: F,
}

impl<F> SqlStoreInMemory<F> {
    pub fn build(initializer: F) -> SqlStoreInMemory<F>
    where
        F: Fn(&Connection) -> Result<()>,
    {
        SqlStoreInMemory { initializer }
    }
}

impl<F> SqlProvider for SqlStoreInMemory<F>
where
    F: Fn(&Connection) -> Result<()>,
{
    fn connection(&self) -> Result<Connection> {
        use diesel::sqlite::SqliteConnection;
        use diesel::Connection as DieselConnection;

        let connection = SqliteConnection::establish(":memory:").expect("in memory database");
        embedded_migrations::run(&connection).expect("create tables");
        (self.initializer)(&connection)?;

        Ok(connection)
    }
}
