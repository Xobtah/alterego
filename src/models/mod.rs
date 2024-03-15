use rusqlite::Connection;

use crate::error::AlterResult;

use self::{
    basic_group_wrapper::BasicGroupWrapper, chat_llm_model::ChatLlmModel,
    chat_wrapper::ChatWrapper, message_wrapper::MessageWrapper,
    supergroup_wrapper::SupergroupWrapper, user_wrapper::UserWrapper,
};

pub mod basic_group_wrapper;
pub mod chat_llm_model;
pub mod chat_wrapper;
pub mod message_wrapper;
pub mod supergroup_wrapper;
pub mod user_wrapper;

pub trait AutoRequestable {
    type UniqueIdentifier;

    fn create_table_request() -> String;
    fn get_id(&self) -> Self::UniqueIdentifier;
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error>
    where
        Self: std::marker::Sized;
    fn select_by_id(
        id: Self::UniqueIdentifier,
        conn: &rusqlite::Connection,
    ) -> AlterResult<Option<Self>>
    where
        Self: std::marker::Sized;
    fn select_all(conn: &rusqlite::Connection) -> AlterResult<Vec<Self>>
    where
        Self: std::marker::Sized;
    fn insert(&self, conn: &rusqlite::Connection) -> AlterResult<()>;
    fn update(&self, conn: &rusqlite::Connection) -> AlterResult<()>;
}

pub fn init_db(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        &BasicGroupWrapper::create_table_request(),
        rusqlite::params![],
    )?;
    conn.execute(&ChatLlmModel::create_table_request(), rusqlite::params![])?;
    conn.execute(&ChatWrapper::create_table_request(), rusqlite::params![])?;
    conn.execute(&MessageWrapper::create_table_request(), rusqlite::params![])?;
    conn.execute(
        &MessageWrapper::create_archive_table_request(),
        rusqlite::params![],
    )?;
    conn.execute(
        &SupergroupWrapper::create_table_request(),
        rusqlite::params![],
    )?;
    conn.execute(&UserWrapper::create_table_request(), rusqlite::params![])?;
    Ok(())
}
