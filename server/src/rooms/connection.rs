use axum::extract::ws::{Message, WebSocket};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use mongodb::bson::{oid::ObjectId, Uuid};
use tokio::{
    select,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::models::user::User;

use super::room::{ClientSentCommand, ConnId, RoomCommands};

pub struct Connection {
    pub commands: Sender<ConnectionCommands>,
    pub id: ConnId,
    pub user: User,
    commands_rx: Receiver<ConnectionCommands>,
    room_commands: Sender<RoomCommands>,
    ws_tx: SplitSink<WebSocket, Message>,
    ws_rx: SplitStream<WebSocket>,
}

#[derive(Debug)]
pub enum ConnectionCommands {
    Send(String),
    Stop,
}

impl Connection {
    pub async fn new(
        ws: WebSocket,
        user: User,
        room_commands: Sender<RoomCommands>,
    ) -> Result<Self, SplitSink<WebSocket, Message>> {
        let (ws_tx, ws_rx) = ws.split();
        let (commands, commands_rx) = mpsc::channel::<ConnectionCommands>(100);
        let id = ConnId(ObjectId::new());

        match room_commands
            .send(RoomCommands::AddConnection(
                id.clone(),
                commands.clone(),
                user.clone(),
            ))
            .await
        {
            Err(err) => {
                log::error!("Erroring adding connection: {}", err);
                return Err(ws_tx);
            }
            _ => (),
        }

        Ok(Self {
            ws_tx,
            ws_rx,
            id,
            user,
            room_commands,
            commands,
            commands_rx,
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

                Some(command) = self.commands_rx.recv() => {
                    if self.handle_command(command).await {
                        break;
                    }
                }
            }
        }
        self.room_commands
            .send(RoomCommands::RemoveConnection(self.id))
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
                self.room_commands
                    .send(RoomCommands::ClientSent(self.id.clone(), data))
                    .await?;
            }
            _ => (),
        }
        Ok(())
    }
}
