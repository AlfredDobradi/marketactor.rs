use crate::actor::{ActorMessage, StorageHandle};
use crate::market;

#[derive(Debug)]
pub struct HistorianActor {
    receiver: tokio::sync::mpsc::Receiver<ActorMessage>,
    storage_handle: StorageHandle,
}

impl HistorianActor {
    pub fn new(receiver: tokio::sync::mpsc::Receiver<ActorMessage>, storage_handle: StorageHandle) -> Self {
        HistorianActor { receiver, storage_handle }
    }

    #[tracing::instrument(skip(self))]
    pub async fn handle_message(&mut self, msg: ActorMessage) {
        match msg {
            ActorMessage::HistoryRequest { region_id, type_id, done } => {
                let client = reqwest::Client::builder()
                    .user_agent("actors/0.1.0")
                    .build()
                    .expect("Failed to build reqwest client");

                let history: Vec<market::HistoryItem> = client.get(format!(
                    "https://esi.evetech.net/latest/markets/{region_id}/history/?datasource=tranquility&type_id={type_id}"
                )).send().await.unwrap().json().await.unwrap();

                let avg = history.iter().fold(0.0, |acc, item| acc + item.average) / history.len() as f64;

                let item = market::AggregatedItem {
                    type_id,
                    average: avg,
                };

                tracing::info!("Fetched history for region: {}, type: {}, average: {:.2}", region_id, type_id, item.average);
                let store_msg = ActorMessage::StoreRequest { item };
                if let Err(e) = self.storage_handle.sender.send(store_msg).await {
                    tracing::error!("Failed to send store request for region: {}, type: {}. Error: {}", region_id, type_id, e);
                }

                let _ = done.send(());
            },
            _ => {}
        }
    }
}

pub async fn spawn_actor(mut actor: HistorianActor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await;
    }
}