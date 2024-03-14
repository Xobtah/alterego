pub type AlterResult<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Tdlib(tdlib::types::Error),
    Io(std::io::Error),
    Database(rusqlite::Error),
    Tokio(tokio::task::JoinError),
    Dialoguer(dialoguer::Error),
    Signal(tokio::sync::broadcast::error::SendError<()>),
    Reqwest(reqwest::Error),
}

impl From<tdlib::types::Error> for Error {
    fn from(e: tdlib::types::Error) -> Self {
        Self::Tdlib(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(e: tokio::task::JoinError) -> Self {
        Self::Tokio(e)
    }
}

impl From<dialoguer::Error> for Error {
    fn from(e: dialoguer::Error) -> Self {
        Self::Dialoguer(e)
    }
}

impl From<tokio::sync::broadcast::error::SendError<()>> for Error {
    fn from(e: tokio::sync::broadcast::error::SendError<()>) -> Self {
        Self::Signal(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}
