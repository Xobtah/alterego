use std::sync::{Arc, Mutex};

use crate::application::Application;
use clap::Parser;
use database::Database;
use error::AlterResult;

mod ai;
mod application;
mod args;
mod database;
mod error;
mod models;
mod ollama;
mod save;
mod update_stream;
mod utils;

#[tokio::main]
async fn main() -> AlterResult<()> {
    env_logger::init();
    let args = args::Args::parse();
    let db = Database::new(&args.database_path)?;
    Application::new(
        include!("../app.id"),
        include_str!("../app.hash"),
        &args.tg_database_directory,
    )
    .run(|update_rx, client_id, mut shutdown_rx| {
        Box::pin(async move {
            let db = Arc::new(Mutex::new(db));
            let (message_tx, message_rx) = tokio::sync::mpsc::unbounded_channel();
            let ai_handle = tokio::spawn(ai::run(
                db.clone(),
                args.model_name,
                message_rx,
                client_id,
                shutdown_rx.resubscribe(),
            ));
            loop {
                tokio::select! {
                    Some((update, client_id)) = update_rx.recv() => {
                        save::update(db.clone(), &update, client_id);
                        match update {
                            tdlib::enums::Update::NewMessage(message) => {
                                if let Err(e) = message_tx.send(message.message) {
                                    log::error!("{e:#?}");
                                }
                            }
                            _ => {}
                        }
                    },
                    _ = shutdown_rx.recv() => {
                        log::info!("Received shutdown signal");
                        break;
                    }
                }
            }
            ai_handle.abort();
            Ok(())
        })
    })
    .await
}
