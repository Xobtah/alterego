use std::ops::Deref;

use rusqlite::OptionalExtension;
use serde::{Serialize, Serializer};
use tdlib::types::BasicGroup;

use crate::error::AlterResult;

use super::AutoRequestable;

#[derive(Debug)]
pub struct BasicGroupWrapper(BasicGroup);

impl Deref for BasicGroupWrapper {
    type Target = BasicGroup;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<BasicGroup> for BasicGroupWrapper {
    fn from(basic_group: BasicGroup) -> Self {
        Self(basic_group)
    }
}

impl Serialize for BasicGroupWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl AutoRequestable for BasicGroupWrapper {
    type UniqueIdentifier = i64;

    fn create_table_request() -> String {
        r#"CREATE TABLE IF NOT EXISTS BASIC_GROUPS (
            id INTEGER PRIMARY KEY,
            member_count INTEGER NOT NULL,
            status TEXT NOT NULL,
            is_active BOOLEAN NOT NULL,
            upgraded_to_supergroup_id INTEGER NOT NULL
        )"#
        .into()
    }

    fn get_id(&self) -> Self::UniqueIdentifier {
        self.0.id
    }

    fn from_row(row: &rusqlite::Row) -> Result<BasicGroupWrapper, rusqlite::Error> {
        Ok(BasicGroupWrapper(BasicGroup {
            id: row.get("id")?,
            member_count: row.get("member_count")?,
            status: serde_json::from_str(&row.get::<_, String>("status")?).unwrap(),
            is_active: row.get("is_active")?,
            upgraded_to_supergroup_id: row.get("upgraded_to_supergroup_id")?,
        }))
    }

    fn select_by_id(
        id: Self::UniqueIdentifier,
        conn: &rusqlite::Connection,
    ) -> AlterResult<Option<Self>> {
        Ok(conn
            .prepare(r#"SELECT * FROM BASIC_GROUPS WHERE id = :id"#)?
            .query_row(
                rusqlite::named_params! {
                    r#":id"#: id,
                },
                Self::from_row,
            )
            .optional()?)
    }

    fn select_all(conn: &rusqlite::Connection) -> AlterResult<Vec<Self>> {
        Ok(conn
            .prepare(r#"SELECT * FROM BASIC_GROUPS"#)?
            .query_map(rusqlite::named_params! {}, Self::from_row)?
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<Self>>())
    }

    fn insert(&self, conn: &rusqlite::Connection) -> AlterResult<()> {
        conn.execute(
            r#"INSERT INTO BASIC_GROUPS (
            id,
            member_count,
            status,
            is_active,
            upgraded_to_supergroup_id
        ) VALUES (
            :id,
            :member_count,
            :status,
            :is_active,
            :upgraded_to_supergroup_id
        )"#
            .into(),
            rusqlite::named_params! {
                ":id": &self.0.id,
                ":member_count": &self.0.member_count,
                ":status": &serde_json::to_string(&self.0.status).unwrap(),
                ":is_active": &self.0.is_active,
                ":upgraded_to_supergroup_id": &self.0.upgraded_to_supergroup_id,
            },
        )?;
        Ok(())
    }

    fn update(&self, conn: &rusqlite::Connection) -> AlterResult<()> {
        conn.execute(
            r#"UPDATE BASIC_GROUPS
            SET
                member_count = :member_count,
                status = :status,
                is_active = :is_active,
                upgraded_to_supergroup_id = :upgraded_to_supergroup_id
            WHERE
                id = :id"#
                .into(),
            rusqlite::named_params! {
                ":id": &self.0.id,
                ":member_count": &self.0.member_count,
                ":status": &serde_json::to_string(&self.0.status).unwrap(),
                ":is_active": &self.0.is_active,
                ":upgraded_to_supergroup_id": &self.0.upgraded_to_supergroup_id,
            },
        )?;
        Ok(())
    }
}
