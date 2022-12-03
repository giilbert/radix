use mongodb::bson::Uuid;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
    time::{self, Duration},
};

use crate::models::user::User;

use super::connection::ConnectionCommands;

#[derive(Debug)]
pub enum RoomCommands {
    Stop,

    AddConnection(ConnId, Sender<ConnectionCommands>, User),
    RemoveConnection(ConnId),

    ClientSent(ConnId, ClientSentCommand),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t", content = "c")]
pub enum ClientSentCommand {
    Ping,
    SendChatMessage { content: String },
}

#[derive(Serialize, Debug)]
#[serde(tag = "t", content = "c")]
pub enum ServerSentCommand {
    // Pong,
    Error(String),
    ChatHistory(VecDeque<ChatMessage>),
    ChatMessage(ChatMessage),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoom {
    pub name: String,
    pub public: bool,
}

#[derive(Debug, Clone)]
pub struct RoomConfig {
    pub name: String,
    pub public: bool,
    pub owner: User,
}

#[derive(Serialize, Debug, Clone)]
pub struct Author {
    name: String,
    id: String,
    is_owner: bool,
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "t", content = "c")]
pub enum ChatMessage {
    UserChat { author: Author, content: String },
    Connection { username: String },
    Disconnection { username: String },
    Bad,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ConnId(pub String);
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct UserId(pub String);

const CHAT_MAX_MESSAGES: usize = 10;

pub struct Room {
    pub commands: Sender<RoomCommands>,
    commands_rx: Receiver<RoomCommands>,
    deletion_timer: Option<JoinHandle<()>>,
    config: RoomConfig,
    connections: HashMap<ConnId, (Sender<ConnectionCommands>, UserId)>,
    users: HashMap<UserId, (ConnId, User)>,
    chat_messages: VecDeque<ChatMessage>,
    pub id: Uuid,
}

impl Room {
    pub fn new(id: Uuid, config: RoomConfig) -> Self {
        let (commands, commands_rx) = mpsc::channel::<RoomCommands>(200);

        Room {
            commands,
            commands_rx,
            config,
            deletion_timer: None,
            connections: Default::default(),
            users: Default::default(),
            chat_messages: Default::default(),
            id,
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        self.prime_deletion();

        log::info!("Room {} running", self.config.name);

        while let Some(msg) = self.commands_rx.recv().await {
            let stop = self.handle_command(msg).await?;
            if stop {
                break;
            }
        }

        log::info!("Room {} stopped", self.config.name);

        Ok(())
    }

    async fn send_connection<T: Serialize>(
        &mut self,
        connection_id: &ConnId,
        data: &T,
    ) -> anyhow::Result<()> {
        let commands = self
            .connections
            .get(connection_id)
            .ok_or_else(|| anyhow::anyhow!("Connection {} not found.", connection_id.0))?;

        commands
            .0
            .send(ConnectionCommands::Send(serde_json::to_string(data)?))
            .await?;

        Ok(())
    }

    async fn handle_command(&mut self, command: RoomCommands) -> anyhow::Result<bool> {
        use RoomCommands::*;

        match command {
            AddConnection(id, commands, user) => {
                self.cancel_deletion();

                log::info!("Room {}: connection {} added", self.config.name, id.0);

                let user_id = UserId(user.id.to_string());
                self.connections
                    .insert(id.clone(), (commands, user_id.clone()));

                let username = user.name.clone();
                self.users.insert(user_id.clone(), (id.clone(), user));

                self.send_connection(
                    &id,
                    &ServerSentCommand::ChatHistory(self.chat_messages.clone()),
                )
                .await?;
                self.send_chat_message(ChatMessage::Connection { username })
                    .await?;
            }
            RemoveConnection(id) => {
                let (_, user_id) = self.connections.remove(&id).ok_or_else(|| {
                    anyhow::anyhow!("Trying to remove an nonexistent connection.")
                })?;
                let (_, user) = self
                    .users
                    .remove(&user_id)
                    .ok_or_else(|| anyhow::anyhow!("Trying to remove an nonexistent user."))?;
                self.send_chat_message(ChatMessage::Disconnection {
                    username: user.name,
                })
                .await?;

                log::info!("Room {}: connection {} removed", self.config.name, id.0);

                if self.connections.len() == 0 {
                    self.prime_deletion();
                }
            }
            ClientSent(id, data) => match data {
                ClientSentCommand::Ping => (),
                ClientSentCommand::SendChatMessage { content } => {
                    let author_id = &self
                        .connections
                        .get(&id)
                        .ok_or_else(|| anyhow::anyhow!("User does not exist"))?
                        .1;
                    let user = &self
                        .users
                        .get(&author_id)
                        .ok_or_else(|| anyhow::anyhow!("User does not exist"))?
                        .1;

                    self.send_chat_message(ChatMessage::UserChat {
                        author: Author {
                            name: user.name.clone(),
                            id: id.0,
                            is_owner: user.id == self.config.owner.id,
                        },
                        content,
                    })
                    .await?;
                }
            },
            Stop => {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn send_all_command(&mut self, command: &ServerSentCommand) -> anyhow::Result<()> {
        let data = serde_json::to_string(command)?;

        for (commands, ..) in self.connections.values() {
            commands
                .send(ConnectionCommands::Send(data.clone()))
                .await?;
        }

        Ok(())
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

    async fn send_chat_message(&mut self, chat_message: ChatMessage) -> anyhow::Result<()> {
        self.chat_messages.push_back(chat_message.clone());
        if self.chat_messages.len() > CHAT_MAX_MESSAGES {
            self.chat_messages.pop_front();
        }
        self.send_all_command(&ServerSentCommand::ChatMessage(chat_message))
            .await?;
        Ok(())
    }
}
