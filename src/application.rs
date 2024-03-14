use std::pin::Pin;

use dialoguer::{theme::ColorfulTheme, Input};
use futures::{Future, StreamExt};
use log::{debug, error, info};
use tdlib::{
    enums::{AuthorizationState, Update},
    functions,
    types::{Message, UpdateAuthorizationState},
};
use tokio::{
    signal::unix,
    sync::{broadcast, mpsc},
};

use crate::{error::AlterResult, update_stream::UpdateStream};

pub struct ApplicationData {
    pub client_id: i32,
    pub auth_rx: mpsc::UnboundedReceiver<AuthorizationState>,
    pub message_rx: mpsc::UnboundedReceiver<Message>,
    pub shutdown_rx: tokio::sync::broadcast::Receiver<()>,
}

unsafe impl Send for ApplicationData {}
unsafe impl Sync for ApplicationData {}

pub struct Application {
    app_id: i32,
    app_hash: String,
    database_name: String,
}

impl Application {
    pub fn new(app_id: i32, app_hash: &str, database_directory: &str) -> Self {
        Self {
            app_id,
            app_hash: app_hash.into(),
            database_name: database_directory.into(),
        }
    }

    pub async fn run<F>(&self, f: F) -> AlterResult<()>
    where
        F: for<'a> FnOnce(
            &'a mut tokio::sync::mpsc::UnboundedReceiver<(Update, i32)>,
            i32,
            tokio::sync::broadcast::Receiver<()>,
        ) -> Pin<Box<dyn Future<Output = AlterResult<()>> + 'a>>,
    {
        let client_id = tdlib::create_client();
        debug!("Client ID '{client_id}' created");

        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        let (update_tx, mut update_rx) = mpsc::unbounded_channel();

        // The update receiver is a separate task that listens for updates from the TDLib client
        // It must be spawned before any other task that sends updates to the client
        let update_handler = tokio::spawn(async move {
            let mut update_stream = UpdateStream;
            loop {
                tokio::select! {
                    Some((update, client_id)) = update_stream.next() => {
                        if let Err(e) = update_tx.send((update, client_id)) {
                            error!("{e:#?}");
                        }
                    }
                }
            }
        });

        let mut sigint = unix::signal(unix::SignalKind::interrupt())?;
        let mut sigterm = unix::signal(unix::SignalKind::terminate())?;
        let mut sighup = unix::signal(unix::SignalKind::hangup())?;
        let mut sigquit = unix::signal(unix::SignalKind::quit())?;
        let signal_handler = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = sigint.recv() => shutdown(&shutdown_tx),
                    _ = sigterm.recv() => shutdown(&shutdown_tx),
                    _ = sighup.recv() => shutdown(&shutdown_tx),
                    _ = sigquit.recv() => shutdown(&shutdown_tx),
                }
            }

            fn shutdown(shutdown_tx: &broadcast::Sender<()>) {
                debug!("Shutdown signal received");
                if let Err(e) = shutdown_tx.send(()) {
                    error!("{e:#?}");
                }
            }
        });

        self.login(&mut update_rx, client_id, shutdown_rx.resubscribe())
            .await?;

        if let Err(e) = f(&mut update_rx, client_id, shutdown_rx.resubscribe()).await {
            error!("{e:#?}");
        }

        self.close(&mut update_rx, client_id, shutdown_rx.resubscribe())
            .await?;

        update_handler.abort();
        signal_handler.abort();
        Ok(())
    }

    async fn login(
        &self,
        update_rx: &mut tokio::sync::mpsc::UnboundedReceiver<(Update, i32)>,
        client_id: i32,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> AlterResult<()> {
        debug!("Logging in");
        functions::set_log_verbosity_level(1, client_id).await?;

        loop {
            tokio::select! {
                Some((update, client_id)) = update_rx.recv() => match update {
                    Update::AuthorizationState(UpdateAuthorizationState { authorization_state }) => match authorization_state {
                        AuthorizationState::WaitTdlibParameters => set_tdlib_parameters(
                            self.app_id,
                            &self.app_hash,
                            &self.database_name,
                            client_id,
                        ).await,
                        AuthorizationState::WaitPhoneNumber => wait_phone_number(client_id).await,
                        AuthorizationState::WaitCode(_) => wait_code(client_id).await,
                        AuthorizationState::WaitPassword(_) => wait_password(client_id).await,
                        AuthorizationState::Ready => break,
                        _ => (),
                    },
                    _ => (),
                },
                _ = shutdown_rx.recv() => {
                    debug!("Received shutdown signal");
                    break;
                },
            }
        }

        debug!("Logged in");
        Ok(())
    }

    async fn close(
        &self,
        update_rx: &mut tokio::sync::mpsc::UnboundedReceiver<(Update, i32)>,
        client_id: i32,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> AlterResult<()> {
        info!("Closing connection");
        functions::close(client_id).await?;

        loop {
            tokio::select! {
                Some((update, _)) = update_rx.recv() => match update {
                    Update::AuthorizationState(UpdateAuthorizationState { authorization_state }) => match authorization_state {
                        AuthorizationState::Closed => break,
                        _ => (),
                    },
                    _ => (),
                },
                _ = shutdown_rx.recv() => {
                    debug!("Received shutdown signal");
                    break;
                }
            }
        }

        info!("Connection closed");
        Ok(())
    }
}

fn ask_user(prompt: &str) -> AlterResult<String> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact_text()?
        .trim()
        .into())
}

async fn set_tdlib_parameters(app_id: i32, app_hash: &str, database_name: &str, client_id: i32) {
    let response = functions::set_tdlib_parameters(
        false,
        database_name.into(),
        Default::default(),
        Default::default(),
        true,
        true,
        true,
        true,
        app_id,
        app_hash.into(),
        "en".into(),
        "Desktop".into(),
        Default::default(),
        env!("CARGO_PKG_VERSION").into(),
        false,
        true,
        client_id,
    )
    .await;

    if let Err(e) = response {
        error!("{e:#?}");
    }
}

async fn wait_phone_number(client_id: i32) {
    while let Err(e) = functions::set_authentication_phone_number(
        ask_user("Enter your phone number (include the country calling code):").unwrap(),
        None,
        client_id,
    )
    .await
    {
        error!("{e:#?}");
    }
}

async fn wait_code(client_id: i32) {
    while let Err(e) = functions::check_authentication_code(
        ask_user("Enter the verification code:").unwrap(),
        client_id,
    )
    .await
    {
        error!("{e:#?}");
    }
}

async fn wait_password(client_id: i32) {
    while let Err(e) = functions::check_authentication_password(
        ask_user("Enter the password:").unwrap(),
        client_id,
    )
    .await
    {
        error!("{e:#?}");
    }
}
