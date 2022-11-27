use fred::prelude::{PubsubInterface, SetsInterface};
use serde::{Deserialize, Serialize};

use crate::{redis::Redis, rooms::room::RoomCommands};

pub struct RoomProxy {
    redis: Redis,
    id: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PartialUser {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct ConnectionIdWithUserId(String, String);

impl RoomProxy {
    pub fn new(redis: Redis, id: &str) -> Self {
        Self {
            redis,
            id: format!("room:{}", id),
        }
    }

    pub async fn send_command(&self, command: &RoomCommands) -> anyhow::Result<()> {
        let _: i32 = self
            .redis
            .publish(&self.id, serde_json::to_string(&command).unwrap())
            .await?;

        Ok(())
    }

    pub async fn add_connection(
        &self,
        connection_id: &String,
        user_id: &String,
    ) -> anyhow::Result<()> {
        let _: i32 = self
            .redis
            .sadd(
                format!("{}:connections", self.id),
                serde_json::to_string(&ConnectionIdWithUserId(
                    connection_id.clone(),
                    user_id.clone(),
                ))?,
            )
            .await?;

        Ok(())
    }

    pub async fn remove_connection(
        &self,
        connection_id: &String,
        user_id: &String,
    ) -> anyhow::Result<()> {
        let _: i32 = self
            .redis
            .srem(
                format!("{}:connections", self.id),
                serde_json::to_string(&ConnectionIdWithUserId(
                    connection_id.clone(),
                    user_id.clone(),
                ))?,
            )
            .await?;

        Ok(())
    }

    pub async fn add_user(&self, data: &PartialUser) -> anyhow::Result<()> {
        let _: i32 = self
            .redis
            .sadd(format!("{}:users", self.id), serde_json::to_string(&data)?)
            .await?;

        Ok(())
    }

    pub async fn does_user_exist(&self, data: &PartialUser) -> anyhow::Result<bool> {
        let exists: bool = self
            .redis
            .sismember(format!("{}:users", self.id), serde_json::to_string(&data)?)
            .await?;

        Ok(exists)
    }

    pub async fn remove_user(&self, data: &PartialUser) -> anyhow::Result<()> {
        let _: i32 = self
            .redis
            .srem(format!("{}:users", self.id), serde_json::to_string(&data)?)
            .await?;

        Ok(())
    }
}
