use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time,
};

use log::{debug, error, info};
use tdlib::{
    enums::{InputMessageContent, MessageContent, MessageReplyTo, MessageSender, User},
    functions,
    types::{FormattedText, InputMessageText, Message, MessageReplyToMessage, MessageSenderUser},
};
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::{
    database::Database,
    error::AlterResult,
    models::{message_wrapper::MessageWrapper, AutoRequestable},
    ollama::{self, OllamaMessage, OllamaRole},
    utils,
};

const READING_WPM_MIN: f64 = 180.;
const READING_WPM_MAX: f64 = 250.;
const THINKING_WPM_MIN: f64 = 1000.;
const THINKING_WPM_MAX: f64 = 3000.;
const TYPING_WPM_MIN: f64 = 80.;
const TYPING_WPM_MAX: f64 = 180.;

pub async fn run(
    db: Arc<Mutex<Database>>,
    model_name: String,
    mut message_rx: mpsc::UnboundedReceiver<Message>,
    client_id: i32,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> AlterResult<()> {
    info!("Start listening for messages");
    let User::User(me) = functions::get_me(client_id).await.unwrap();

    let mut thoughts: HashMap<i64, oneshot::Sender<oneshot::Sender<()>>> = HashMap::new();

    loop {
        tokio::select! {
            Some(message) = message_rx.recv() => {
                let failsafe = {
                    // Skip messages from me
                    let _user_id = match message.sender_id {
                        MessageSender::User(MessageSenderUser { user_id }) => if user_id == me.id {
                            continue;
                        } else {
                            info!(
                                "[{}] {}: {}",
                                utils::chat_display_name(db.clone(), message.chat_id),
                                utils::user_display_name(db.clone(), user_id),
                                message_text(&message).unwrap_or_else(|| "(Not text)".into()),
                            );
                            user_id
                        },
                        _ => continue,
                    };

                    if message.chat_id < 0 {
                        if let Some(usernames) = &me.usernames {
                            if !message_text(&message).unwrap_or_default().contains(&format!("@{}", usernames.editable_username)) {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    }

                    if let Some(interrupt_tx) = thoughts.remove(&message.chat_id) {
                        if !interrupt_tx.is_closed() {
                            let (interrupt_ack_tx, interrupt_ack_rx) = tokio::sync::oneshot::channel();
                            interrupt_tx.send(interrupt_ack_tx).unwrap();
                            let _ = interrupt_ack_rx.await;
                        }
                    }

                    let (interrupt_tx, interrupt_rx) = tokio::sync::oneshot::channel();
                    let chat_id = message.chat_id;

                    if message.chat_id < 0 {
                        tokio::spawn(handle_group_message(model_name.clone(), message, client_id, interrupt_rx));
                    } else {
                        tokio::spawn(handle_private_message(db.clone(), model_name.clone(), me.id, message, client_id, interrupt_rx));
                    }

                    thoughts.insert(chat_id, interrupt_tx);

                    Ok(())
                } as AlterResult<()>;

                if let Err(e) = failsafe {
                    error!("{e:#?}");
                }
            },
            _ = shutdown_rx.recv() => {
                debug!("Received shutdown signal");
                break;
            }
        }
    }

    info!("Stop listening for messages");
    Ok(())
}

async fn handle_group_message(
    model_name: String,
    message: Message,
    client_id: i32,
    interrupt_rx: tokio::sync::oneshot::Receiver<tokio::sync::oneshot::Sender<()>>,
) -> i64 {
    let chat_id = message.chat_id;
    debug!("[{chat_id}] Handling message");
    let mut handle = tokio::spawn(async move {
        let now = time::Instant::now();
        let text = message_text(&message).unwrap_or_else(|| "Salut".into());
        let response = ollama::request(&model_name, &text).await?;
        simulate_waiting(
            &text,
            &response,
            now.elapsed(),
            message.chat_id,
            message.message_thread_id,
            client_id,
        )
        .await?;
        send_message(message, response.clone(), client_id).await?;
        Ok(()) as AlterResult<()>
    });

    tokio::select! {
        task_result = &mut handle => match task_result {
            Ok(Ok(_)) => {},
            Ok(Err(e)) => error!("[{chat_id}] Failed to handle message: {e:#?}"),
            Err(e) => error!("[{chat_id}] Task failure: {e:#?}"),
        },
        Ok(interrupt_ack_tx) = interrupt_rx => {
            handle.abort();
            debug!("[{chat_id}] Interrupted");
            let _ = interrupt_ack_tx.send(());
        },
    }
    chat_id
}

async fn handle_private_message(
    db: Arc<Mutex<Database>>,
    model_name: String,
    me_id: i64,
    message: Message,
    client_id: i32,
    interrupt_rx: tokio::sync::oneshot::Receiver<tokio::sync::oneshot::Sender<()>>,
) -> i64 {
    let chat_id = message.chat_id;
    debug!("[{chat_id}] Handling message");
    let mut handle = tokio::spawn(async move {
        functions::view_messages(
            message.chat_id,
            vec![message.id],
            Some(tdlib::enums::MessageSource::Other),
            true,
            client_id,
        )
        .await?;
        let now = time::Instant::now();
        let response = ollama::chat(
            &model_name,
            &get_ollama_conversation(db.clone(), me_id, message.chat_id)?,
        )
        .await?;
        simulate_waiting(
            &message_text(&message).unwrap_or_else(|| "Salut".into()),
            &response.content,
            now.elapsed(),
            message.chat_id,
            message.message_thread_id,
            client_id,
        )
        .await?;
        send_message(message, response.content.clone(), client_id).await?;
        Ok(()) as AlterResult<()>
    });

    tokio::select! {
        task_result = &mut handle => match task_result {
            Ok(Ok(_)) => {},
            Ok(Err(e)) => error!("[{chat_id}] Failed to handle message: {e:#?}"),
            Err(e) => error!("[{chat_id}] Task failure: {e:#?}"),
        },
        Ok(interrupt_ack_tx) = interrupt_rx => {
            handle.abort();
            debug!("[{chat_id}] Interrupted");
            let _ = interrupt_ack_tx.send(());
        },
    }
    chat_id
}

fn get_ollama_conversation(
    db: Arc<Mutex<Database>>,
    assistant_id: i64,
    chat_id: i64,
) -> AlterResult<Vec<OllamaMessage>> {
    Ok(db
        .lock()
        .unwrap()
        .execute(|conn| {
            Ok(conn
                .prepare("SELECT * FROM MESSAGES WHERE chat_id = ?1")?
                .query_map(
                    rusqlite::params![chat_id],
                    <MessageWrapper as AutoRequestable>::from_row,
                )?
                .filter_map(Result::ok)
                .collect::<Vec<MessageWrapper>>())
        })?
        .into_iter()
        .map(<MessageWrapper as Into<Message>>::into)
        .map(|message| OllamaMessage {
            role: match message.sender_id {
                MessageSender::User(MessageSenderUser { user_id }) => {
                    if user_id == assistant_id {
                        OllamaRole::Assistant
                    } else {
                        OllamaRole::User
                    }
                }
                MessageSender::Chat(_) => OllamaRole::System,
            },
            content: message_text(&message).unwrap_or_else(|| "Salut".into()),
        })
        .collect())
}

fn message_text(message: &Message) -> Option<String> {
    match &message.content {
        MessageContent::MessageText(text) => Some(text.text.text.clone()),
        MessageContent::MessagePhoto(photo) => Some(photo.caption.text.clone()),
        MessageContent::MessageVideo(video) => Some(video.caption.text.clone()),
        MessageContent::MessageAnimation(animation) => Some(animation.caption.text.clone()),
        _ => None,
    }
}

async fn simulate_waiting(
    message: &str,
    answer: &str,
    elapsed: time::Duration,
    chat_id: i64,
    message_thread_id: i64,
    client_id: i32,
) -> AlterResult<()> {
    let word_delimiters = [
        ' ', '\n', '\t', '\r', ',', '.', '!', '?', ':', ';', '(', ')', '"', '\'',
    ];
    let words_number =
        |s: &str| (s.chars().filter(|c| word_delimiters.contains(c)).count() + 1) as f64;
    let min_max_wait = |w_nb, (min, max)| (w_nb / max * 60. * 1000., w_nb / min * 60. * 1000.);

    let (reading_min, reading_max) =
        min_max_wait(words_number(message), (READING_WPM_MIN, READING_WPM_MAX));
    let (thinking_min, thinking_max) =
        min_max_wait(words_number(answer), (THINKING_WPM_MIN, THINKING_WPM_MAX));
    utils::sleep_ms(
        utils::rand_between(reading_min as u64, reading_max as u64)
            + utils::rand_between(thinking_min as u64, thinking_max as u64),
    )
    .await;

    let (typing_min, typing_max) =
        min_max_wait(words_number(answer), (TYPING_WPM_MIN, TYPING_WPM_MAX));
    let typing_wait = utils::rand_between(typing_min as u64, typing_max as u64);
    let mut typing = std::pin::pin!(utils::sleep_ms(
        if typing_wait < elapsed.as_millis() as u64 {
            0
        } else {
            typing_wait - elapsed.as_millis() as u64
        }
    ));
    loop {
        tokio::select! {
            _ = &mut typing => break,
            _ = functions::send_chat_action(
                chat_id,
                message_thread_id,
                Some(tdlib::enums::ChatAction::Typing),
                client_id,
            ) => {
                let _ = utils::sleep_ms(5000).await;
            },
        }
    }

    Ok(())
}

async fn send_message(message: Message, text: String, client_id: i32) -> AlterResult<()> {
    info!("Sending message");
    functions::send_message(
        message.chat_id,
        message.message_thread_id,
        if message.chat_id < 0 {
            Some(MessageReplyTo::Message(MessageReplyToMessage {
                chat_id: message.chat_id,
                message_id: message.id,
            }))
        } else {
            None
        },
        None,
        InputMessageContent::InputMessageText(InputMessageText {
            text: FormattedText {
                text,
                entities: vec![],
            },
            disable_web_page_preview: true,
            clear_draft: false,
        }),
        client_id,
    )
    .await?;
    Ok(())
}
