use async_broadcast::{broadcast, Sender, Receiver, InactiveReceiver};

#[derive(Debug, Clone)]
pub struct BroadcastChannel<T> {
    sender: Sender<T>,
    inactive_template: InactiveReceiver<T>,
}

impl<T: Clone> BroadcastChannel<T> {
    pub fn new(buffer: usize) -> Self {
        let (sender, receiver) = broadcast(buffer);
        Self {
            sender,
            // deactivate the initial receiver so it does not back up the buffer
            inactive_template: receiver.deactivate(),
        }
    }

    pub fn subscribe(&self) -> Receiver<T> {
        self.inactive_template.clone().activate()
    }

    pub async fn send(&self, value: T) {
        let _ = self.sender.broadcast(value).await;
    }
    
    pub fn send_blocking(&self, value: T) {
        let _ = self.sender.broadcast_blocking(value);
    }

    pub fn spawn_send(&self, value: T)
    where
        T: 'static + Send,
    {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let _ = sender.broadcast(value).await;
        });
    }
}