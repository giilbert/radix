use axum::extract::ws::{Message, WebSocket};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use mongodb::bson::Uuid;
use serde::{Deserialize, Serialize};
use tokio::select;

use crate::{
    models::user::User,
    redis::{
        channels::{ChannelReceiver, ChannelsExt},
        Redis,
    },
};

use super::{
    proxies::room::RoomProxy,
    room::{ClientSentCommand, RoomCommands},
};

pub struct Connection {
    ws_tx: SplitSink<WebSocket, Message>,
    ws_rx: SplitStream<WebSocket>,
    redis_channel: ChannelReceiver,
    broadcast_channel: ChannelReceiver,
    room: RoomProxy,
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
    ) -> Result<Self, (WebSocket, anyhow::Error)> {
        let id = format!("connection:{}", Uuid::new().to_string());
        let redis_channel = match redis.listen(id.clone()).await {
            Ok(chan) => chan,
            Err(err) => return Err((ws, err)),
        };
        let broadcast_channel = match redis.listen(format!("room:{}:broadcast", room_name)).await {
            Ok(chan) => chan,
            Err(err) => return Err((ws, err)),
        };
        let (ws_tx, ws_rx) = ws.split();

        let room = RoomProxy::new(redis, &room_name);

        Ok(Self {
            ws_tx,
            ws_rx,
            redis_channel,
            broadcast_channel,
            id,
            room,
            user,
        })
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
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
                Some(redis_command) = self.broadcast_channel.recv() => {
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

        self.room
            .send_command(&RoomCommands::RemoveConnection(
                self.id,
                self.user.id.to_string(),
                self.user.name,
            ))
            .await?;

        Ok(())
    }

    pub async fn handle_command(&mut self, command: ConnectionCommands) -> bool {
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
                let data = serde_json::from_str::<ClientSentCommand>(&string)?;
                self.room
                    .send_command(&RoomCommands::ClientSent(self.id.clone(), data))
                    .await?;
            }
            _ => (),
        }

        Ok(())
    }
}
