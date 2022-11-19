use fred::prelude::{KeysInterface, SetsInterface, TransactionInterface};
use serde::{Deserialize, Serialize};
use tokio::{
    select,
    sync::mpsc,
    task::JoinHandle,
    time::{self, Duration},
};

use crate::{models::user::User, redis::Redis};

pub enum RoomCommands {
    Ping,
    Stop,
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
    name: String,
    public: bool,
    owner: User,
    rx: mpsc::Receiver<RoomCommands>,
    commands: mpsc::Sender<RoomCommands>,
    redis: Redis,
}

impl Room {
    pub fn new(redis: &Redis, data: CreateRoom, owner: User) -> Self {
        let (tx, rx) = mpsc::channel::<RoomCommands>(256);

        Room {
            name: data.name,
            public: data.public,
            owner,
            rx,
            commands: tx,
            redis: redis.clone(),
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        let commands = self.commands.clone();
        let mut sleep_handle = Some(tokio::spawn(async move {
            const FIVE_MINUTES: u64 = 10;
            time::sleep(Duration::from_secs(FIVE_MINUTES)).await;
            if let Err(err) = commands.send(RoomCommands::Stop).await {
                log::error!("Error stopping room: {}", err);
            };
        }));

        let redis = self.redis;

        {
            let t = redis.multi(true).await?;
            let _ = t.sadd::<i32, _, _>("rooms", &self.name).await;
            let _ = t
                .set::<i32, _, _>(
                    format!("room-{}", &self.name),
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

        while let Some(command) = self.rx.recv().await {
            use RoomCommands::*;

            match command {
                Ping => log::info!("a"),
                Stop => {
                    break;
                }
            }
        }

        // cleanup
        {
            let t = redis.multi(true).await?;
            let _ = t.srem::<i32, _, _>("rooms", &self.name).await;
            let _ = t.del::<i32, _>(format!("room-{}", &self.name)).await;
            t.exec().await?;
        }

        log::info!("Room {} stopped", self.name);

        Ok(())
    }
}
