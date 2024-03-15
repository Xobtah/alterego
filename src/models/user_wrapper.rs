use std::ops::Deref;

use rusqlite::OptionalExtension;
use serde::{Serialize, Serializer};
use tdlib::types::User;

use crate::error::AlterResult;

use super::AutoRequestable;

#[derive(Debug)]
pub struct UserWrapper(User);

impl Deref for UserWrapper {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<User> for UserWrapper {
    fn from(user: User) -> Self {
        Self(user)
    }
}

impl Into<User> for UserWrapper {
    fn into(self) -> User {
        self.0
    }
}

impl Serialize for UserWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl AutoRequestable for UserWrapper {
    type UniqueIdentifier = i64;

    fn create_table_request() -> String {
        r#"CREATE TABLE IF NOT EXISTS USERS (
            id INTEGER PRIMARY KEY,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            usernames TEXT,
            phone_number TEXT NOT NULL,
            status TEXT NOT NULL,
            profile_photo TEXT,
            emoji_status TEXT,
            is_contact BOOLEAN NOT NULL,
            is_mutual_contact BOOLEAN NOT NULL,
            is_close_friend BOOLEAN NOT NULL,
            is_verified BOOLEAN NOT NULL,
            is_premium BOOLEAN NOT NULL,
            is_support BOOLEAN NOT NULL,
            restriction_reason TEXT,
            is_scam BOOLEAN NOT NULL,
            is_fake BOOLEAN NOT NULL,
            has_active_stories BOOLEAN NOT NULL,
            has_unread_active_stories BOOLEAN NOT NULL,
            have_access BOOLEAN NOT NULL,
            user_type TEXT NOT NULL,
            language_code TEXT NOT NULL,
            added_to_attachment_menu BOOLEAN NOT NULL
        )"#
        .into()
    }

    fn get_id(&self) -> Self::UniqueIdentifier {
        self.0.id
    }

    fn from_row(row: &rusqlite::Row) -> Result<UserWrapper, rusqlite::Error> {
        Ok(UserWrapper(User {
            id: row.get("id")?,
            first_name: row.get("first_name")?,
            last_name: row.get("last_name")?,
            usernames: serde_json::from_str(&row.get::<_, String>("usernames")?).unwrap(),
            phone_number: row.get("phone_number")?,
            status: serde_json::from_str(&row.get::<_, String>("status")?).unwrap(),
            profile_photo: serde_json::from_str(&row.get::<_, String>("profile_photo")?).unwrap(),
            emoji_status: serde_json::from_str(&row.get::<_, String>("emoji_status")?).unwrap(),
            is_contact: row.get("is_contact")?,
            is_mutual_contact: row.get("is_mutual_contact")?,
            is_close_friend: row.get("is_close_friend")?,
            is_verified: row.get("is_verified")?,
            is_premium: row.get("is_premium")?,
            is_support: row.get("is_support")?,
            restriction_reason: row.get("restriction_reason")?,
            is_scam: row.get("is_scam")?,
            is_fake: row.get("is_fake")?,
            has_active_stories: row.get("has_active_stories")?,
            has_unread_active_stories: row.get("has_unread_active_stories")?,
            have_access: row.get("have_access")?,
            r#type: serde_json::from_str(&row.get::<_, String>("user_type")?).unwrap(),
            language_code: row.get("language_code")?,
            added_to_attachment_menu: row.get("added_to_attachment_menu")?,
        }))
    }

    fn select_by_id(
        id: Self::UniqueIdentifier,
        conn: &rusqlite::Connection,
    ) -> AlterResult<Option<Self>> {
        Ok(conn
            .prepare(r#"SELECT * FROM USERS WHERE id = :id"#)?
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
            .prepare(r#"SELECT * FROM USERS"#)?
            .query_map(rusqlite::named_params! {}, Self::from_row)?
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<Self>>())
    }

    fn insert(&self, conn: &rusqlite::Connection) -> AlterResult<()> {
        conn.execute(
            r#"INSERT INTO USERS (
            id,
            first_name,
            last_name,
            usernames,
            phone_number,
            status,
            profile_photo,
            emoji_status,
            is_contact,
            is_mutual_contact,
            is_close_friend,
            is_verified,
            is_premium,
            is_support,
            restriction_reason,
            is_scam,
            is_fake,
            has_active_stories,
            has_unread_active_stories,
            have_access,
            user_type,
            language_code,
            added_to_attachment_menu
        ) VALUES (
            :id,
            :first_name,
            :last_name,
            :usernames,
            :phone_number,
            :status,
            :profile_photo,
            :emoji_status,
            :is_contact,
            :is_mutual_contact,
            :is_close_friend,
            :is_verified,
            :is_premium,
            :is_support,
            :restriction_reason,
            :is_scam,
            :is_fake,
            :has_active_stories,
            :has_unread_active_stories,
            :have_access,
            :user_type,
            :language_code,
            :added_to_attachment_menu
        )"#
            .into(),
            rusqlite::named_params! {
                ":id": &self.0.id,
                ":first_name": &self.0.first_name,
                ":last_name": &self.0.last_name,
                ":usernames": &serde_json::to_string(&self.0.usernames).unwrap(),
                ":phone_number": &self.0.phone_number,
                ":status": &serde_json::to_string(&self.0.status).unwrap(),
                ":profile_photo": &serde_json::to_string(&self.0.profile_photo).unwrap(),
                ":emoji_status": &serde_json::to_string(&self.0.emoji_status).unwrap(),
                ":is_contact": &self.0.is_contact,
                ":is_mutual_contact": &self.0.is_mutual_contact,
                ":is_close_friend": &self.0.is_close_friend,
                ":is_verified": &self.0.is_verified,
                ":is_premium": &self.0.is_premium,
                ":is_support": &self.0.is_support,
                ":restriction_reason": &self.0.restriction_reason,
                ":is_scam": &self.0.is_scam,
                ":is_fake": &self.0.is_fake,
                ":has_active_stories": &self.0.has_active_stories,
                ":has_unread_active_stories": &self.0.has_unread_active_stories,
                ":have_access": &self.0.have_access,
                ":user_type": &serde_json::to_string(&self.0.r#type).unwrap(),
                ":language_code": &self.0.language_code,
                ":added_to_attachment_menu": &self.0.added_to_attachment_menu,
            },
        )?;
        Ok(())
    }

    fn update(&self, conn: &rusqlite::Connection) -> AlterResult<()> {
        conn.execute(
            r#"UPDATE USERS
            SET
                first_name = :first_name,
                last_name = :last_name,
                usernames = :usernames,
                phone_number = :phone_number,
                status = :status,
                profile_photo = :profile_photo,
                emoji_status = :emoji_status,
                is_contact = :is_contact,
                is_mutual_contact = :is_mutual_contact,
                is_close_friend = :is_close_friend,
                is_verified = :is_verified,
                is_premium = :is_premium,
                is_support = :is_support,
                restriction_reason = :restriction_reason,
                is_scam = :is_scam,
                is_fake = :is_fake,
                has_active_stories = :has_active_stories,
                has_unread_active_stories = :has_unread_active_stories,
                have_access = :have_access,
                user_type = :user_type,
                language_code = :language_code,
                added_to_attachment_menu = :added_to_attachment_menu
            WHERE
                id = :id"#
                .into(),
            rusqlite::named_params! {
                ":id": &self.0.id,
                ":first_name": &self.0.first_name,
                ":last_name": &self.0.last_name,
                ":usernames": &serde_json::to_string(&self.0.usernames).unwrap(),
                ":phone_number": &self.0.phone_number,
                ":status": &serde_json::to_string(&self.0.status).unwrap(),
                ":profile_photo": &serde_json::to_string(&self.0.profile_photo).unwrap(),
                ":emoji_status": &serde_json::to_string(&self.0.emoji_status).unwrap(),
                ":is_contact": &self.0.is_contact,
                ":is_mutual_contact": &self.0.is_mutual_contact,
                ":is_close_friend": &self.0.is_close_friend,
                ":is_verified": &self.0.is_verified,
                ":is_premium": &self.0.is_premium,
                ":is_support": &self.0.is_support,
                ":restriction_reason": &self.0.restriction_reason,
                ":is_scam": &self.0.is_scam,
                ":is_fake": &self.0.is_fake,
                ":has_active_stories": &self.0.has_active_stories,
                ":has_unread_active_stories": &self.0.has_unread_active_stories,
                ":have_access": &self.0.have_access,
                ":user_type": &serde_json::to_string(&self.0.r#type).unwrap(),
                ":language_code": &self.0.language_code,
                ":added_to_attachment_menu": &self.0.added_to_attachment_menu,
            },
        )?;
        Ok(())
    }
}
