use std::sync::{Arc, Mutex};

use log::{debug, error, trace};
use tdlib::{
    enums::{MessageContent, Update},
    functions,
    types::{Message, UpdateDeleteMessages},
};

use crate::{
    database::Database,
    error::AlterResult,
    models::{
        basic_group_wrapper::BasicGroupWrapper, chat_wrapper::ChatWrapper,
        message_wrapper::MessageWrapper, supergroup_wrapper::SupergroupWrapper,
        user_wrapper::UserWrapper,
    },
};

pub fn update(db: Arc<Mutex<Database>>, update: &Update, client_id: i32) {
    let result = match update {
        Update::NewMessage(message) => {
            let db = db.lock().unwrap();
            download_message_content(&message.message, client_id);
            db.save(&MessageWrapper::from(message.message.clone()))
        }
        Update::DeleteMessages(UpdateDeleteMessages {
            from_cache: false,
            message_ids,
            ..
        }) => {
            debug!("Archiving {} messages: {message_ids:?}", message_ids.len());
            let db = db.lock().unwrap();
            archive_messages(&db, &message_ids);
            Ok(())
        }
        Update::NewChat(tdlib::types::UpdateNewChat { chat }) => {
            if let Some(photo) = &chat.photo {
                download_file(photo.big.id, client_id);
            }
            let db = db.lock().unwrap();
            db.save(&ChatWrapper::from(chat.clone()))
        }
        Update::Supergroup(tdlib::types::UpdateSupergroup { supergroup }) => {
            let db = db.lock().unwrap();
            db.save(&SupergroupWrapper::from(supergroup.clone()))
        }
        Update::BasicGroup(tdlib::types::UpdateBasicGroup { basic_group }) => {
            let db = db.lock().unwrap();
            db.save(&BasicGroupWrapper::from(basic_group.clone()))
        }
        Update::User(tdlib::types::UpdateUser { user }) => {
            if let Some(photo) = &user.profile_photo {
                download_file(photo.big.id, client_id);
            }
            let db = db.lock().unwrap();
            db.save(&UserWrapper::from(user.clone()))
        }
        _ => Ok(()),
    };

    if let Err(e) = result {
        error!("{e:#?}");
    }
}

fn archive_messages(db: &Database, message_ids: &[i64]) {
    for message_id in message_ids {
        match db.load::<MessageWrapper>(*message_id) {
            Ok(Some(message)) => {
                if let Err(e) = db.execute(|conn| {
                    message.delete(conn)?;
                    Ok(vec![]) as AlterResult<Vec<MessageWrapper>>
                }) {
                    error!("{e:#?}");
                }
            }
            Ok(None) => {}
            Err(e) => error!("{e:#?}"),
        }
    }
}

fn download_message_content(message: &Message, client_id: i32) {
    match &message.content {
        MessageContent::MessagePhoto(message_photo) => {
            if let Some(photo) = message_photo
                .photo
                .sizes
                .iter()
                .max_by(|a, b| (a.width * a.height).cmp(&(b.width * b.height)))
            {
                download_file(photo.photo.id, client_id);
            }
        }
        MessageContent::MessageVideo(message_video) => {
            download_file(message_video.video.video.id, client_id);
        }
        MessageContent::MessageAnimation(message_animation) => {
            download_file(message_animation.animation.animation.id, client_id);
        }
        _ => {}
    }
}

fn download_file(file_id: i32, client_id: i32) {
    tokio::spawn(async move {
        if let Err(e) = functions::download_file(file_id, 1, 0, 0, true, client_id).await {
            error!("{e:#?}");
        } else {
            trace!("Downloaded file: {file_id}");
        }
    });
}
