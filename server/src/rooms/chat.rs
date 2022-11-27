use fred::prelude::{ListInterface, PubsubInterface, TransactionInterface};
use serde::{Deserialize, Serialize};

use crate::redis::Redis;

use super::{
    connection::ConnectionCommands,
    proxies::room::RoomProxy,
    room::{RoomCommands, ServerSentCommand},
};

#[derive(Deserialize, Serialize, Debug)]
pub struct Author {
    name: String,
    id: String,
    is_owner: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "t", content = "c")]
pub enum ChatMessage {
    UserChat { author: Author, content: String },
    Connection { username: String },
    Disconnection { username: String },
    Bad,
}

pub struct Chat {
    chat_id: String,
    room_id: String,
    redis: Redis,
}

impl Chat {
    pub fn new(redis: Redis, room_id: String) -> Self {
        let chat_id = format!("{}:chat", room_id);
        Self {
            chat_id,
            room_id,
            redis,
        }
    }

    pub async fn new_message(&self, message: ChatMessage) -> anyhow::Result<()> {
        let t = self.redis.multi(true).await?;
        let _ = t
            .lpush::<i32, _, _>(&self.chat_id, serde_json::to_string(&message)?)
            .await;
        let _ = t.ltrim::<i32, _>(&self.chat_id, 0, 100).await;
        t.exec().await?;

        self.redis
            .publish::<i32, _, _>(
                format!("{}:broadcast", self.room_id),
                serde_json::to_string(&ConnectionCommands::Send(serde_json::to_string(
                    &ServerSentCommand::ChatMessage(message),
                )?))?,
            )
            .await?;

        Ok(())
    }

    pub async fn get_all_messages(&self) -> anyhow::Result<Vec<ChatMessage>> {
        Ok(self
            .redis
            .lrange::<Vec<String>, _>(&self.chat_id, 0, 100)
            .await?
            .iter()
            .map(|s| serde_json::from_str::<ChatMessage>(s.as_str()).unwrap_or(ChatMessage::Bad))
            .collect::<Vec<ChatMessage>>())
    }
}
