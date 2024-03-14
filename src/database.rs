use std::path::Path;

use log::debug;
use rusqlite::Connection;

use crate::{
    error::AlterResult,
    models::{init_db, AutoRequestable},
};

pub struct Database {
    conn: Connection,
}

unsafe impl Send for Database {}
unsafe impl Sync for Database {}

impl Database {
    pub fn new(db_path: &Path) -> Result<Self, rusqlite::Error> {
        debug!("Creating database '{}'", db_path.display());
        let conn = Connection::open(db_path)?;
        init_db(&conn)?;
        Ok(Self { conn })
    }

    pub fn save<DatabaseEntity: AutoRequestable + serde::Serialize>(
        &self,
        entity: &DatabaseEntity,
    ) -> AlterResult<()> {
        if let Some(_) = DatabaseEntity::select_by_id(entity.get_id(), &self.conn)? {
            entity.update(&self.conn)?;
        } else {
            entity.insert(&self.conn)?;
        }
        Ok(())
    }

    pub fn load<DatabaseEntity: AutoRequestable<UniqueIdentifier = i64>>(
        &self,
        id: i64,
    ) -> AlterResult<Option<DatabaseEntity>> {
        DatabaseEntity::select_by_id(id, &self.conn)
    }

    pub fn load_all<DatabaseEntity: AutoRequestable>(&self) -> AlterResult<Vec<DatabaseEntity>> {
        DatabaseEntity::select_all(&self.conn)
    }

    pub fn execute<DatabaseEntity: AutoRequestable>(
        &self,
        context: impl Fn(&Connection) -> AlterResult<Vec<DatabaseEntity>>,
    ) -> AlterResult<Vec<DatabaseEntity>> {
        context(&self.conn)
    }
}
