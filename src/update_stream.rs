use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;
use tdlib::enums::Update;

pub struct UpdateStream;

impl Stream for UpdateStream {
    type Item = (Update, i32);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match tdlib::receive() {
            Some((update, client_id)) => Poll::Ready(Some((update, client_id))),
            None => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}
