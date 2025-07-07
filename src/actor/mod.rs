use tokio::{sync::oneshot, task::JoinHandle};
use tracing::debug;
use rand::prelude::*;

use crate::{actor::storage::StorageActor, market};

pub mod historian;
pub mod storage;

#[derive(Debug)]
pub enum ActorMessage {
    HistoryRequest {
        region_id: i64,
        type_id: i64,
        done: oneshot::Sender<()>,
    },
    StoreRequest {
        item: market::AggregatedItem,
    },
    GetItemRequest {
        type_id: i64,
    }
}

#[derive(Clone)]
pub struct Manager {
    pub handles: Vec<HistorianHandle>,
    pub storage_handle: StorageHandle,
}

impl Manager {
    pub fn build(num: i64) -> Self {
        let db = storage::Db::default();
        let storage_handle = StorageHandle::new(db);
        let mut handles = Vec::with_capacity(num as usize);

        for _ in 0..num {
            let (handle, _join_handle) = HistorianHandle::new(storage_handle.clone());
            handles.push(handle);
        }

        Self {
            handles,
            storage_handle,
        }
    }

    pub async fn send_history_request(
        &mut self,
        region_id: i64,
        type_id: i64,
    ) {
        let actor = self.handles.choose_mut(&mut rand::rng()).unwrap();
        actor.send_history_request(region_id, type_id).await;
    }
}

#[derive(Clone)]
pub struct HistorianHandle {
    id: uuid::Uuid,
    sender: tokio::sync::mpsc::Sender<ActorMessage>,
}

impl HistorianHandle {
    pub fn new(storage_handle: StorageHandle) -> (Self, JoinHandle<()>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(8);
        let actor = historian::HistorianActor::new(receiver, storage_handle);
        let handle = tokio::spawn(historian::spawn_actor(actor));

        (Self { id: uuid::Uuid::new_v4(), sender }, handle)
    }

    #[tracing::instrument(skip(self), fields(id = %self.id))]
    pub async fn send_history_request(
        &self,
        region_id: i64,
        type_id: i64,
    ) {
        let (done_tx, done_rx) = oneshot::channel();
        let msg = ActorMessage::HistoryRequest {
            region_id,
            type_id,
            done: done_tx,
        };
        debug!("Sending history request");
        let _ = self.sender.send(msg).await;
        let _ = done_rx.await;
    }
}

#[derive(Clone, Debug)]
pub struct StorageHandle {
    sender: tokio::sync::mpsc::Sender<ActorMessage>,
}

impl StorageHandle {
    pub fn new(db: storage::Db) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(8);
        let actor = StorageActor::new(receiver, db);
        tokio::spawn(storage::spawn_actor(actor));

        Self { sender }
    }

    pub async fn send_store_request(
        &self,
        item: market::AggregatedItem,
    ) {
        let msg = ActorMessage::StoreRequest { item };
        let _ = self.sender.send(msg).await;
    }
}