use axum::async_trait;
use core::ops::Deref;
use fred::{prelude::PubsubInterface, types::RedisValue};
use futures_util::StreamExt;
use std::{collections::HashMap, ops::DerefMut, sync::Arc};
use tokio::sync::{mpsc, RwLock};

use super::Redis;

pub struct ChannelReceiver {
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
        tokio::task::spawn(async move {
            channel_table.write().await.0.remove(&key);
            if redis.unsubscribe(&key).await.is_err() {
                log::error!("Error unsubscribing from channel: {}", key);
            }
        });
    }
}

#[derive(Default)]
pub struct ChannelTable(pub HashMap<String, mpsc::Sender<RedisValue>>);

#[async_trait]
pub trait ChannelsExt {
    async fn listen(&self, name: String) -> anyhow::Result<ChannelReceiver>;
    async fn run_channels(self);
}

#[async_trait]
impl ChannelsExt for Redis {
    async fn listen(&self, name: String) -> anyhow::Result<ChannelReceiver> {
        let (tx, rx) = mpsc::channel::<RedisValue>(256);
        self.subscribe(&name).await?;

        if self
            .channel_table
            .write()
            .await
            .0
            .insert(name.clone(), tx)
            .is_some()
        {
            anyhow::bail!("Channel listener already exists.");
        };

        Ok(ChannelReceiver {
            inner: rx,
            key: name,
            channel_table: self.channel_table.clone(),
            redis: self.clone(),
        })
    }

    async fn run_channels(self) {
        while let Some((channel_name, message)) = self.on_message().next().await {
            let channel_table = self.channel_table.read().await;
            if let Some(channel) = channel_table.0.get(&channel_name) {
                if channel.send(message).await.is_err() {
                    log::error!("Error pushing message to channel {}", channel_name);
                }
            } else {
                log::error!("Channel {} does not exist.", channel_name)
            }
        }
    }
}
