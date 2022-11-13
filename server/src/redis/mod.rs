pub mod channels;

use core::ops::{Deref, DerefMut};
use fred::{
    prelude::{ClientLike, RedisClient},
    types::{ReconnectPolicy, RedisConfig},
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::redis::channels::ChannelsExt;

#[derive(Clone)]
pub struct Redis {
    client: RedisClient,
    channel_table: Arc<RwLock<channels::ChannelTable>>,
}

impl Redis {
    pub async fn connect() -> anyhow::Result<Self> {
        let config = RedisConfig::default();
        let client = RedisClient::new(config);
        let reconnect_policy = ReconnectPolicy::default();
        let _ = client.connect(Some(reconnect_policy));
        let _ = client.wait_for_connect().await?;

        log::info!("Connected to Redis.");

        let channel_table = Arc::new(RwLock::new(channels::ChannelTable::default()));

        let this = Self {
            client,
            channel_table,
        };
        tokio::task::spawn(this.clone().run_channels());

        Ok(this)
    }
}

impl Deref for Redis {
    type Target = RedisClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for Redis {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}
