use std::sync::mpsc;

use super::LockfreeMutCell;

/// Simple wrapper around `std::mpsc` channels except receiver does not need to be mutable.
/// Obviously not safe to call `recv` from multiple threads, it's a single consumer channel duh
pub struct SyncChannel<T> {
    pub tx: mpsc::Sender<T>,
    pub rx: LockfreeMutCell<mpsc::Receiver<T>>,
}

pub struct SenderDropped;

impl<T> SyncChannel<T> {
    pub fn new() -> Self {
        Self::from_tx_rx(mpsc::channel())
    }

    fn from_tx_rx((tx, rx): (mpsc::Sender<T>, mpsc::Receiver<T>)) -> Self {
        Self {
            tx,
            rx: LockfreeMutCell::new(rx),
        }
    }

    pub fn send(&self, msg: T) -> Result<(), mpsc::SendError<T>> {
        self.tx.send(msg)
    }

    /// Safety: is guaranteed to be safe as long as you don't call it from multiple threads at once.
    pub unsafe fn recv(&self) -> Result<T, SenderDropped> {
        let chan = self.rx.get_mut();
        chan.recv().map_err(|_| SenderDropped)
    }
}

impl<T> Default for SyncChannel<T> {
    fn default() -> Self {
        Self::new()
    }
}
