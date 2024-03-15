use std::{
    sync::{Arc, Mutex},
    time,
};

use log::debug;
use rand::Rng;
use tdlib::{enums::MessageContent, types::Message};

use crate::{
    database::Database,
    models::{chat_wrapper::ChatWrapper, user_wrapper::UserWrapper},
};

pub fn rand_between(min: u64, max: u64) -> u64 {
    let (min, max) = if min > max { (max, min) } else { (min, max) };
    (rand::thread_rng().gen::<f64>() * (max - min) as f64) as u64 + min
}

pub async fn sleep_ms(waiting_time: u64) {
    debug!("Waiting for {waiting_time} ms");
    tokio::time::sleep(time::Duration::from_millis(waiting_time)).await;
}

pub fn user_display_name(db: Arc<Mutex<Database>>, user_id: i64) -> String {
    db.lock()
        .unwrap()
        .load::<UserWrapper>(user_id)
        .and_then(|user| {
            Ok(user.map(|user| {
                let user = <UserWrapper as Into<tdlib::types::User>>::into(user);
                format!("{} {}", user.first_name, user.last_name)
                    .trim()
                    .into()
            }))
        })
        .unwrap_or_default()
        .unwrap_or_else(|| user_id.to_string())
}

pub fn chat_display_name(db: Arc<Mutex<Database>>, chat_id: i64) -> String {
    db.lock()
        .unwrap()
        .load::<ChatWrapper>(chat_id)
        .and_then(|chat| {
            Ok(chat.map(|chat| <ChatWrapper as Into<tdlib::types::Chat>>::into(chat).title))
        })
        .unwrap_or_default()
        .unwrap_or_else(|| chat_id.to_string())
}

pub fn message_text(message: &Message) -> Option<String> {
    match &message.content {
        MessageContent::MessageText(text) => Some(text.text.text.clone()),
        MessageContent::MessagePhoto(photo) => Some(photo.caption.text.clone()),
        MessageContent::MessageVideo(video) => Some(video.caption.text.clone()),
        MessageContent::MessageAnimation(animation) => Some(animation.caption.text.clone()),
        _ => None,
    }
}
