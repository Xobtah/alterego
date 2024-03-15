use rusqlite::OptionalExtension;

use crate::error::AlterResult;

use super::AutoRequestable;

#[derive(Debug)]
pub struct ChatLlmModel(i64, String);

impl ChatLlmModel {
    pub fn chat_id(&self) -> i64 {
        self.0
    }

    pub fn model_name(&self) -> &str {
        &self.1
    }
}

impl AutoRequestable for ChatLlmModel {
    type UniqueIdentifier = i64;

    fn create_table_request() -> String {
        r#"CREATE TABLE IF NOT EXISTS CHAT_LLM_MODELS (
            chat_id INTEGER PRIMARY KEY,
            model_name TEXT NOT NULL
        )"#
        .into()
    }

    fn get_id(&self) -> Self::UniqueIdentifier {
        self.chat_id()
    }

    fn from_row(row: &rusqlite::Row) -> Result<ChatLlmModel, rusqlite::Error> {
        Ok(ChatLlmModel(row.get("chat_id")?, row.get("model_name")?))
    }

    fn select_by_id(
        id: Self::UniqueIdentifier,
        conn: &rusqlite::Connection,
    ) -> AlterResult<Option<Self>> {
        Ok(conn
            .prepare(r#"SELECT chat_id, model_name FROM CHAT_LLM_MODELS WHERE chat_id = :chat_id"#)?
            .query_row(
                rusqlite::named_params! {
                    r#":chat_id"#: id,
                },
                Self::from_row,
            )
            .optional()?)
    }

    fn select_all(conn: &rusqlite::Connection) -> AlterResult<Vec<Self>> {
        Ok(conn
            .prepare(r#"SELECT * FROM CHAT_LLM_MODELS"#)?
            .query_map(rusqlite::named_params! {}, Self::from_row)?
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<Self>>())
    }

    fn insert(&self, conn: &rusqlite::Connection) -> AlterResult<()> {
        conn.execute(
            r#"INSERT INTO CHAT_LLM_MODELS (
            chat_id,
            model_name
        ) VALUES (
            :chat_id,
            :model_name
        )"#
            .into(),
            rusqlite::named_params! {
                ":chat_id": &self.chat_id(),
                ":model_name": self.model_name(),
            },
        )?;
        Ok(())
    }

    fn update(&self, conn: &rusqlite::Connection) -> AlterResult<()> {
        conn.execute(
            r#"UPDATE CHAT_LLM_MODELS
            SET
                model_name = :model_name
            WHERE
                chat_id = :chat_id"#
                .into(),
            rusqlite::named_params! {
                ":chat_id": &self.chat_id(),
                ":model_name": self.model_name(),
            },
        )?;
        Ok(())
    }
}
