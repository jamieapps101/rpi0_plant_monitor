pub enum Event {
    Tick,
}

use tokio::{sync::mpsc::Sender,time::sleep};
use std::time::Duration;
pub async fn ticker(event_sink: Sender<Event>, period: u64) {
    loop {
        // do sleep first to allow initial connection
        sleep(Duration::from_secs(period)).await;
        if event_sink.send(Event::Tick).await.is_err() {
            panic!("Could not send tick signal")
        }
    }
}