use fred::prelude::{KeysInterface, PubsubInterface, SetsInterface, TransactionInterface};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tokio::{
    select,
    sync::mpsc::{self, Sender},
    task::JoinHandle,
    time::{self, Duration},
};

use crate::{
    models::user::User,
    redis::{channels::ChannelsExt, Redis},
    rooms::proxies::room::PartialUser,
};

use super::{
    chat::{Chat, ChatMessage},
    connection::ConnectionCommands,
    proxies::room::RoomProxy,
};

#[derive(Serialize, Deserialize, Debug)]
pub enum RoomCommands {
    Stop,

    // (connection_id, user_id, name)
    AddConnection(String, String, String),
    RemoveConnection(String, String, String),

    ClientSent(String, ClientSentCommand),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t", content = "c")]
pub enum ClientSentCommand {
    Ping,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t", content = "c")]
pub enum ServerSentCommand {
    Pong,
    ChatHistory(Vec<ChatMessage>),
    ChatMessage(ChatMessage),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoom {
    pub name: String,
    pub public: bool,
}

#[derive(Serialize)]
struct RoomData {
    name: String,
    public: bool,
    owner_id: String,
}

pub struct Room {
    pub name: String,
    pub public: bool,
    pub owner: User,
    pub commands: Sender<RoomCommands>,
    chat: Chat,
    rx: mpsc::Receiver<RoomCommands>,
    redis: Redis,
    proxy: RoomProxy,
    // conn_id => (user_id, name)
    connections: HashMap<String, (String, String)>,
    deletion_timer: Option<JoinHandle<()>>,
}

impl Room {
    pub fn new(redis: &Redis, data: CreateRoom, owner: User) -> Self {
        let (commands, rx) = mpsc::channel::<RoomCommands>(256);
        let chat = Chat::new(redis.clone(), format!("room:{}", data.name));

        Room {
            proxy: RoomProxy::new(redis.clone(), &data.name),
            name: data.name,
            public: data.public,
            owner,
            commands,
            chat,
            rx,
            redis: redis.clone(),
            connections: HashMap::new(),
            deletion_timer: None,
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        self.prime_deletion();
        {
            let t = self.redis.multi(true).await?;
            let _ = t.sadd::<i32, _, _>("rooms", &self.name).await;
            let _ = t
                .set::<i32, _, _>(
                    format!("room:{}", &self.name),
                    serde_json::to_string(&RoomData {
                        public: self.public,
                        name: self.name.clone(),
                        owner_id: self.owner.id.to_string(),
                    })?,
                    None,
                    None,
                    false,
                )
                .await;

            t.exec().await?;
        }

        let mut chan = self.redis.listen(format!("room:{}", self.name)).await?;
        loop {
            select! {
                Some(command) = chan.recv() => {
                    let command = serde_json::from_str::<RoomCommands>(
                        command
                            .as_str()
                            .ok_or_else(|| anyhow::anyhow!("Command send not deserializable"))?
                            .as_ref(),
                    )?;

                    if self.handle_command(command).await? {
                        break;
                    }
                }

                Some(command) = self.rx.recv() => {
                    if self.handle_command(command).await? {
                        break;
                    }
                }
            }
        }

        // cleanup
        {
            let t = self.redis.multi(true).await?;
            let _ = t.srem::<i32, _, _>("rooms", &self.name).await;
            let _ = t.del::<i32, _>(format!("room:{}", &self.name)).await;
            let _ = t.del::<i32, _>(format!("room:{}:chat", &self.name)).await;
            let _ = t.del::<i32, _>(format!("room:{}:users", &self.name)).await;
            let _ = t
                .del::<i32, _>(format!("room:{}:connections", &self.name))
                .await;
            t.exec().await?;
        }

        // stop all connections

        log::info!("Room {} stopped", self.name);

        Ok(())
    }

    async fn send_connection<T: Serialize>(
        &mut self,
        connection_id: &String,
        data: T,
    ) -> anyhow::Result<()> {
        self.redis
            .publish(
                connection_id,
                serde_json::to_string(&ConnectionCommands::Send(serde_json::to_string(&data)?))?,
            )
            .await?;

        Ok(())
    }

    async fn handle_command(&mut self, command: RoomCommands) -> anyhow::Result<bool> {
        use RoomCommands::*;

        match command {
            AddConnection(connection_id, user_id, name) => {
                self.proxy.add_connection(&connection_id, &user_id).await?;
                self.proxy
                    .add_user(&PartialUser {
                        id: user_id.clone(),
                        name: name.clone(),
                    })
                    .await?;

                self.send_connection(
                    &connection_id,
                    ServerSentCommand::ChatHistory(self.chat.get_all_messages().await?),
                )
                .await?;

                self.chat
                    .new_message(ChatMessage::Connection {
                        username: name.clone(),
                    })
                    .await?;

                self.connections.insert(connection_id, (user_id, name));
                self.cancel_deletion();
            }
            RemoveConnection(connection_id, user_id, name) => {
                self.proxy
                    .remove_connection(&connection_id, &user_id)
                    .await?;
                self.proxy
                    .remove_user(&PartialUser {
                        id: user_id,
                        name: name.clone(),
                    })
                    .await?;

                self.connections.remove(&connection_id);
                self.chat
                    .new_message(ChatMessage::Disconnection { username: name })
                    .await?;

                if self.connections.len() == 0 {
                    self.prime_deletion();
                }
            }
            ClientSent(id, data) => match data {
                ClientSentCommand::Ping => {
                    self.send_connection(&id, "Pong").await?;
                }
            },
            Stop => {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn prime_deletion(&mut self) {
        if self.deletion_timer.is_none() {
            let commands = self.commands.clone();
            self.deletion_timer = Some(tokio::spawn(async move {
                const FIVE_MINUTES: u64 = 30;
                time::sleep(Duration::from_secs(FIVE_MINUTES)).await;
                if let Err(err) = commands.send(RoomCommands::Stop).await {
                    log::error!("Error stopping room: {}", err);
                };
            }));
        }
    }

    fn cancel_deletion(&mut self) {
        if let Some(task) = self.deletion_timer.take() {
            task.abort();
        }
    }

    async fn broadcast<T: Serialize>(&self, data: T) -> anyhow::Result<()> {
        self.redis
            .publish(
                format!("room:{}:broadcast", self.name),
                serde_json::to_string(&ConnectionCommands::Send(serde_json::to_string(&data)?))?,
            )
            .await?;
        Ok(())
    }
}
