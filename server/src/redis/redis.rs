use crate::redis::channels;
use async_recursion::async_recursion;
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
};
use core::ops::{Deref, DerefMut};
use fred::{
    prelude::{ClientLike, RedisClient},
    types::{ReconnectPolicy, RedisConfig},
};
use reqwest::StatusCode;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::redis::channels::ChannelsExt;

#[derive(Clone)]
pub struct Redis {
    client: RedisClient,
    pub channel_table: Option<Arc<RwLock<channels::ChannelTable>>>,
    pub channels_redis: Option<Arc<Redis>>,
}

impl Redis {
    #[async_recursion]
    pub async fn connect(is_chan: bool) -> anyhow::Result<Self> {
        let config = RedisConfig::default();
        let client = RedisClient::new(config);
        let reconnect_policy = ReconnectPolicy::default();
        let _ = client.connect(Some(reconnect_policy));
        let _ = client.wait_for_connect().await?;

        let channels_redis = if !is_chan {
            let channels_redis = Arc::new(Redis::connect(true).await?);
            tokio::task::spawn(channels_redis.as_ref().clone().run_channels());
            log::info!("Connected to Redis. [channel]");

            Some(channels_redis)
        } else {
            log::info!("Connected to Redis. [main]");
            None
        };
        let channel_table = if is_chan {
            Some(Arc::new(RwLock::new(channels::ChannelTable::default())))
        } else {
            None
        };

        let this = Self {
            client,
            channel_table,
            channels_redis,
        };

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

#[async_trait]
impl<B: Send> FromRequest<B> for Redis {
    type Rejection = StatusCode;
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let redis = req
            .extensions()
            .get::<Redis>()
            .expect("redis not in extensions");

        Ok(redis.clone())
    }
}
