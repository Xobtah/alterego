use std::ops::Deref;

use rusqlite::OptionalExtension;
use serde::{Serialize, Serializer};
use tdlib::types::Supergroup;

use crate::error::AlterResult;

use super::AutoRequestable;

#[derive(Debug)]
pub struct SupergroupWrapper(Supergroup);

impl Deref for SupergroupWrapper {
    type Target = Supergroup;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Supergroup> for SupergroupWrapper {
    fn from(supergroup: Supergroup) -> Self {
        Self(supergroup)
    }
}

impl Serialize for SupergroupWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl AutoRequestable for SupergroupWrapper {
    type UniqueIdentifier = i64;

    fn create_table_request() -> String {
        r#"CREATE TABLE IF NOT EXISTS SUPERGROUPS (
            id INTEGER PRIMARY KEY,
            usernames TEXT,
            date INTEGER NOT NULL,
            status TEXT NOT NULL,
            member_count INTEGER NOT NULL,
            has_linked_chat BOOLEAN NOT NULL,
            has_location BOOLEAN NOT NULL,
            sign_messages BOOLEAN NOT NULL,
            join_to_send_messages BOOLEAN NOT NULL,
            join_by_request BOOLEAN NOT NULL,
            is_slow_mode_enabled BOOLEAN NOT NULL,
            is_channel BOOLEAN NOT NULL,
            is_broadcast_group BOOLEAN NOT NULL,
            is_forum BOOLEAN NOT NULL,
            is_verified BOOLEAN NOT NULL,
            restriction_reason TEXT NOT NULL,
            is_scam BOOLEAN NOT NULL,
            is_fake BOOLEAN NOT NULL,
            has_active_stories BOOLEAN NOT NULL,
            has_unread_active_stories BOOLEAN NOT NULL
        )"#
        .into()
    }

    fn get_id(&self) -> Self::UniqueIdentifier {
        self.0.id
    }

    fn from_row(row: &rusqlite::Row) -> Result<SupergroupWrapper, rusqlite::Error> {
        Ok(SupergroupWrapper(Supergroup {
            id: row.get("id")?,
            usernames: serde_json::from_str(&row.get::<_, String>("usernames")?).unwrap(),
            date: row.get("date")?,
            status: serde_json::from_str(&row.get::<_, String>("status")?).unwrap(),
            member_count: row.get("member_count")?,
            has_linked_chat: row.get("has_linked_chat")?,
            has_location: row.get("has_location")?,
            sign_messages: row.get("sign_messages")?,
            join_to_send_messages: row.get("join_to_send_messages")?,
            join_by_request: row.get("join_by_request")?,
            is_slow_mode_enabled: row.get("is_slow_mode_enabled")?,
            is_channel: row.get("is_channel")?,
            is_broadcast_group: row.get("is_broadcast_group")?,
            is_forum: row.get("is_forum")?,
            is_verified: row.get("is_verified")?,
            restriction_reason: row.get("restriction_reason")?,
            is_scam: row.get("is_scam")?,
            is_fake: row.get("is_fake")?,
            has_active_stories: row.get("has_active_stories")?,
            has_unread_active_stories: row.get("has_unread_active_stories")?,
        }))
    }

    fn select_by_id(
        id: Self::UniqueIdentifier,
        conn: &rusqlite::Connection,
    ) -> AlterResult<Option<Self>> {
        Ok(conn
            .prepare(r#"SELECT * FROM SUPERGROUPS WHERE id = :id"#)?
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
            .prepare(r#"SELECT * FROM SUPERGROUPS"#)?
            .query_map(rusqlite::named_params! {}, Self::from_row)?
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<Self>>())
    }

    fn insert(&self, conn: &rusqlite::Connection) -> AlterResult<()> {
        conn.execute(
            r#"INSERT INTO SUPERGROUPS (
            id,
            usernames,
            date,
            status,
            member_count,
            has_linked_chat,
            has_location,
            sign_messages,
            join_to_send_messages,
            join_by_request,
            is_slow_mode_enabled,
            is_channel,
            is_broadcast_group,
            is_forum,
            is_verified,
            restriction_reason,
            is_scam,
            is_fake,
            has_active_stories,
            has_unread_active_stories
        ) VALUES (
            :id,
            :usernames,
            :date,
            :status,
            :member_count,
            :has_linked_chat,
            :has_location,
            :sign_messages,
            :join_to_send_messages,
            :join_by_request,
            :is_slow_mode_enabled,
            :is_channel,
            :is_broadcast_group,
            :is_forum,
            :is_verified,
            :restriction_reason,
            :is_scam,
            :is_fake,
            :has_active_stories,
            :has_unread_active_stories
        )"#
            .into(),
            rusqlite::named_params! {
                ":id": &self.0.id,
                ":usernames": &serde_json::to_string(&self.0.usernames).unwrap(),
                ":date": &self.0.date,
                ":status": &serde_json::to_string(&self.0.status).unwrap(),
                ":member_count": &self.0.member_count,
                ":has_linked_chat": &self.0.has_linked_chat,
                ":has_location": &self.0.has_location,
                ":sign_messages": &self.0.sign_messages,
                ":join_to_send_messages": &self.0.join_to_send_messages,
                ":join_by_request": &self.0.join_by_request,
                ":is_slow_mode_enabled": &self.0.is_slow_mode_enabled,
                ":is_channel": &self.0.is_channel,
                ":is_broadcast_group": &self.0.is_broadcast_group,
                ":is_forum": &self.0.is_forum,
                ":is_verified": &self.0.is_verified,
                ":restriction_reason": &self.0.restriction_reason,
                ":is_scam": &self.0.is_scam,
                ":is_fake": &self.0.is_fake,
                ":has_active_stories": &self.0.has_active_stories,
                ":has_unread_active_stories": &self.0.has_unread_active_stories,
            },
        )?;
        Ok(())
    }

    fn update(&self, conn: &rusqlite::Connection) -> AlterResult<()> {
        conn.execute(
            r#"UPDATE SUPERGROUPS
            SET
                usernames = :usernames,
                date = :date,
                status = :status,
                member_count = :member_count,
                has_linked_chat = :has_linked_chat,
                has_location = :has_location,
                sign_messages = :sign_messages,
                join_to_send_messages = :join_to_send_messages,
                join_by_request = :join_by_request,
                is_slow_mode_enabled = :is_slow_mode_enabled,
                is_channel = :is_channel,
                is_broadcast_group = :is_broadcast_group,
                is_forum = :is_forum,
                is_verified = :is_verified,
                restriction_reason = :restriction_reason,
                is_scam = :is_scam,
                is_fake = :is_fake,
                has_active_stories = :has_active_stories,
                has_unread_active_stories = :has_unread_active_stories
            WHERE
                id = :id"#
                .into(),
            rusqlite::named_params! {
                ":id": &self.0.id,
                ":usernames": &serde_json::to_string(&self.0.usernames).unwrap(),
                ":date": &self.0.date,
                ":status": &serde_json::to_string(&self.0.status).unwrap(),
                ":member_count": &self.0.member_count,
                ":has_linked_chat": &self.0.has_linked_chat,
                ":has_location": &self.0.has_location,
                ":sign_messages": &self.0.sign_messages,
                ":join_to_send_messages": &self.0.join_to_send_messages,
                ":join_by_request": &self.0.join_by_request,
                ":is_slow_mode_enabled": &self.0.is_slow_mode_enabled,
                ":is_channel": &self.0.is_channel,
                ":is_broadcast_group": &self.0.is_broadcast_group,
                ":is_forum": &self.0.is_forum,
                ":is_verified": &self.0.is_verified,
                ":restriction_reason": &self.0.restriction_reason,
                ":is_scam": &self.0.is_scam,
                ":is_fake": &self.0.is_fake,
                ":has_active_stories": &self.0.has_active_stories,
                ":has_unread_active_stories": &self.0.has_unread_active_stories,
            },
        )?;
        Ok(())
    }
}
