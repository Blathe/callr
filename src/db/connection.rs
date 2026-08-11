use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

use crate::db::error::DbError;
use crate::db::schema::MIGRATIONS_SQL;

pub type SharedConnection = Arc<Mutex<Connection>>;

/// Opens (creating if needed) `<collection_root>/.callr/history.db` and
/// brings it up to the latest schema. A single mutex-guarded connection is
/// sufficient here — this is a single-user app with modest write volume.
pub fn open(collection_root: &Path) -> Result<SharedConnection, DbError> {
    let dir = collection_root.join(".callr");
    std::fs::create_dir_all(&dir)?;
    let db_path = dir.join("history.db");

    let mut conn = Connection::open(db_path)?;
    let migrations = Migrations::new(vec![M::up(MIGRATIONS_SQL)]);
    migrations.to_latest(&mut conn)?;

    Ok(Arc::new(Mutex::new(conn)))
}
