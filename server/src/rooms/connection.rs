use axum::extract::ws::{Message, WebSocket};
use either::Either;
use fred::prelude::PubsubInterface;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use mongodb::bson::Uuid;
use serde::{Deserialize, Serialize};
use tokio::{select, sync::mpsc};

use crate::{
    models::user::User,
    redis::{
        channels::{ChannelReceiver, ChannelsExt},
        Redis,
    },
};

use super::room::{ClientCommand, Room, RoomCommands};

pub struct Connection {
    ws_tx: SplitSink<WebSocket, Message>,
    ws_rx: SplitStream<WebSocket>,
    redis: Redis,
    room_name: String,
    redis_channel: ChannelReceiver,
    pub id: String,
    pub user: User,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ConnectionCommands {
    Send(String),
    Stop,
}

impl Connection {
    pub async fn new(
        ws: WebSocket,
        user: User,
        room_name: String,
        redis: Redis,
    ) -> anyhow::Result<Self> {
        let id = Uuid::new();
        let redis_channel = redis.listen(format!("connection:{}", id)).await?;
        let (ws_tx, ws_rx) = ws.split();

        Ok(Self {
            ws_tx,
            ws_rx,
            redis,
            room_name,
            redis_channel,
            id: format!("connection:{}", id),
            user,
        })
    }

    pub async fn run(mut self) {
        loop {
            select! {
                data = &mut self.ws_rx.next() => {
                    let data = match data {
                        Some(Ok(data)) => data,
                        Some(Err(err)) => {
                            log::error!("Error receiving message: {:?}", err);
                            continue;
                        }
                        None => break
                    };

                    // log::info!("received message {:?}", data);

                    if let Err(err) = self.handle_message(data).await {
                        log::error!("Error handling message {}", err);
                    }
                }
                Some(redis_command) = self.redis_channel.recv() => {
                    use fred::types::RedisValue::*;
                    match redis_command {
                        String(data) => {
                            match serde_json::from_str::<ConnectionCommands>(data.to_string().as_str()) {
                                Ok(data) => {
                                    if self.handle_command(data).await {
                                        break;
                                    }
                                },
                                Err(err) => log::error!("Error receiving message from Redis: {:?}", err),
                            }
                        },
                        _ => continue
                    }
                }
            }
        }

        let _: i32 = self
            .redis
            .publish(
                format!("room:{}", self.room_name),
                serde_json::to_string(&RoomCommands::RemoveConnection(self.id.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    async fn handle_command(&mut self, command: ConnectionCommands) -> bool {
        use ConnectionCommands::*;

        match command {
            Stop => return true,
            Send(data) => {
                if let Err(err) = self.ws_tx.send(Message::Text(data)).await {
                    log::error!("Error sending WebSocket message: {}", err);
                }
            }
        }

        false
    }

    async fn handle_message(&mut self, message: Message) -> anyhow::Result<()> {
        match message {
            Message::Text(string) => {
                let data = serde_json::from_str::<ClientCommand>(&string)?;
                self.redis
                    .publish(
                        format!("room:{}", self.room_name),
                        serde_json::to_string(&RoomCommands::ClientSent(
                            self.id.to_string(),
                            data,
                        ))?,
                    )
                    .await?
            }
            _ => (),
        }

        Ok(())
    }
}
