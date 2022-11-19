use fred::prelude::{KeysInterface, PubsubInterface, SetsInterface, TransactionInterface};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::{
    select,
    sync::mpsc::{self, Sender},
    task::JoinHandle,
    time::{self, Duration},
};

use crate::{
    models::user::User,
    redis::{channels::ChannelsExt, Redis},
};

use super::connection::ConnectionCommands;

#[derive(Serialize, Deserialize, Debug)]
pub enum RoomCommands {
    Ping,
    Stop,

    // ids
    AddConnection(String),
    RemoveConnection(String),

    ClientSent(String, ClientSentCommand),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientSentCommand {
    Ping,
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
    rx: mpsc::Receiver<RoomCommands>,
    redis: Redis,
    connections: HashSet<String>,
    deletion_timer: Option<JoinHandle<()>>,
}

impl Room {
    pub fn new(redis: &Redis, data: CreateRoom, owner: User) -> Self {
        let (commands, rx) = mpsc::channel::<RoomCommands>(256);

        Room {
            name: data.name,
            public: data.public,
            owner,
            commands,
            rx,
            redis: redis.clone(),
            connections: HashSet::new(),
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
            Ping => log::info!("a"),
            AddConnection(id) => {
                self.connections.insert(id);
                self.cancel_deletion();
            }
            RemoveConnection(id) => {
                self.connections.remove(&id);
                if self.connections.len() == 0 {
                    self.prime_deletion();
                }
            }
            Stop => {
                return Ok(true);
            }
            ClientSent(id, data) => match data {
                ClientSentCommand::Ping => {
                    self.send_connection(&id, "Pong").await?;
                }
            },
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
}
