use axum::async_trait;
use core::ops::Deref;
use fred::{prelude::PubsubInterface, types::RedisValue};
use futures_util::StreamExt;
use mongodb::bson::Uuid;
use std::{
    collections::{HashMap, HashSet},
    ops::DerefMut,
    sync::Arc,
};
use tokio::sync::{mpsc, RwLock};

use super::Redis;

pub struct ChannelReceiver {
    id: Uuid,
    inner: mpsc::Receiver<RedisValue>,
    channel_table: Arc<RwLock<ChannelTable>>,
    key: String,
    redis: Redis,
}

impl Deref for ChannelReceiver {
    type Target = mpsc::Receiver<RedisValue>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ChannelReceiver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Drop for ChannelReceiver {
    fn drop(&mut self) {
        let key = self.key.clone();
        let channel_table = self.channel_table.clone();
        let redis = self.redis.clone();
        let id = self.id.clone();

        tokio::task::spawn(async move {
            let mut chan_lock = channel_table.write().await;
            let listeners = chan_lock
                .0
                .get_mut(&key)
                .expect("channel key does not exist");
            listeners.remove(&id);

            if listeners.len() == 0 {
                if let Err(err) = redis.unsubscribe(&key).await {
                    log::error!("Error unsubscribing from channel {}: {:?}", key, err);
                }
            }
        });
    }
}

#[derive(Default)]
pub struct ChannelTable(pub HashMap<String, HashMap<Uuid, mpsc::Sender<RedisValue>>>);

#[async_trait]
pub trait ChannelsExt {
    async fn listen(&self, name: String) -> anyhow::Result<ChannelReceiver>;
    async fn run_channels(self);
}

#[async_trait]
impl ChannelsExt for Redis {
    async fn listen(&self, name: String) -> anyhow::Result<ChannelReceiver> {
        let (tx, rx) = mpsc::channel::<RedisValue>(256);
        let channel_redis = self.channels_redis.as_ref().unwrap();
        let channel_table = channel_redis.channel_table.as_ref().unwrap();
        let id = Uuid::new();

        channel_redis.subscribe(&name).await?;

        let mut table_lock = channel_table.write().await;
        if let Some(channels) = table_lock.0.get_mut(&name) {
            channels.insert(id.clone(), tx);
        } else {
            let mut new = HashMap::default();
            new.insert(id.clone(), tx);
            table_lock.0.insert(name.clone(), new);
        }

        Ok(ChannelReceiver {
            id,
            inner: rx,
            key: name,
            channel_table: channel_table.clone(),
            redis: channel_redis.as_ref().clone(),
        })
    }

    async fn run_channels(self) {
        let channel_table = self.channel_table.as_ref().unwrap();
        while let Some((channel_name, message)) = self.on_message().next().await {
            let channel_table = channel_table.read().await;
            if let Some(channels) = channel_table.0.get(&channel_name) {
                for channel in channels.values() {
                    if channel.send(message.clone()).await.is_err() {
                        log::error!("Error pushing message to channel {}", channel_name);
                    }
                }
            } else {
                log::error!("Channel {} does not exist.", channel_name)
            }
        }
    }
}
