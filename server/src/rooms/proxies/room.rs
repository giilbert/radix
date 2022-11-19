use fred::prelude::PubsubInterface;

use crate::{redis::Redis, rooms::room::RoomCommands};

pub struct RoomProxy {
    redis: Redis,
    id: String,
}

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
}
