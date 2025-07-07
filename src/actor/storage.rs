use std::{collections::HashMap, sync::{Arc, Mutex}};

use tracing::info;

use crate::{actor::ActorMessage, market::AggregatedItem};

pub type Db = Arc<Mutex<HashMap<i64, AggregatedItem>>>;

#[derive(Debug)]
pub struct StorageActor {
    receiver: tokio::sync::mpsc::Receiver<ActorMessage>,
    db: Db,
}

impl StorageActor {
    pub fn new(receiver: tokio::sync::mpsc::Receiver<ActorMessage>, db: Db) -> Self {
        StorageActor { receiver, db }
    }

    pub async fn handle_message(&mut self, msg: ActorMessage) {
        match msg {
            ActorMessage::StoreRequest { item } => {
                info!("Storing item: {:?}", item);
            },
            ActorMessage::GetItemRequest { type_id } => {
                info!("Retrieving item with type_id: {}", type_id);
                let db = Db::default();
                let db_lock = db.lock().unwrap();
                if let Some(item) = db_lock.get(&type_id) {
                    info!("Found item: {:?}", item);
                } else {
                    info!("Item with type_id {} not found", type_id);
                }
            },
            _ => {}
        }
    }
}

pub async fn spawn_actor(mut actor: StorageActor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await;
    }
}