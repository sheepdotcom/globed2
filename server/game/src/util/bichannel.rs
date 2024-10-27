use super::{SenderDropped, TokioChannel};
use crate::tokio::sync::mpsc;

pub struct BiChannel<T> {
    pub up: TokioChannel<T>,
    pub down: TokioChannel<T>,
}

// Bidirectional mpsc channel
impl<T> BiChannel<T> {
    pub fn new(size: usize) -> Self {
        let up = TokioChannel::new(size);
        let down = TokioChannel::new(size);
        Self::from_channels(up, down)
    }

    pub fn from_channels(up: TokioChannel<T>, down: TokioChannel<T>) -> Self {
        Self { up, down }
    }

    pub fn try_send_up(&self, msg: T) -> Result<(), mpsc::error::TrySendError<T>> {
        self.up.try_send(msg)
    }

    pub async fn send_up(&self, msg: T) -> Result<(), mpsc::error::SendError<T>> {
        self.up.send(msg).await
    }

    pub async fn try_send_down(&self, msg: T) -> Result<(), mpsc::error::TrySendError<T>> {
        self.down.try_send(msg)
    }

    pub async fn send_down(&self, msg: T) -> Result<(), mpsc::error::SendError<T>> {
        self.down.send(msg).await
    }

    /// Safety: is guaranteed to be safe as long as you don't call it from multiple threads at once.
    pub async unsafe fn recv_up(&self) -> Result<T, SenderDropped> {
        self.up.recv().await
    }

    /// Safety: is guaranteed to be safe as long as you don't call it from multiple threads at once.
    pub async unsafe fn recv_down(&self) -> Result<T, SenderDropped> {
        self.down.recv().await
    }
}
