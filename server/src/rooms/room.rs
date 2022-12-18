use mongodb::bson::{oid::ObjectId, Uuid};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
    time::{self, Duration},
};

use crate::{
    models::{
        problem::{Code, Problem, ProblemPublic, TestCase},
        user::User,
    },
    rooms::judge,
};

use super::{connection::ConnectionCommands, judge::FailedTestCase};

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
    SendChatMessage {
        content: String,
    },
    BeginRound,
    SetEditorContent {
        content: String,
    },
    TestCode {
        #[serde(rename = "testCases")]
        test_cases: Vec<TestCase>,
        language: String,
    },
    SubmitCode,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublicUser {
    id: String,
    name: String,
    image: String,
}

#[derive(Serialize, Debug)]
#[serde(tag = "t", content = "c")]
pub enum ServerSentCommand {
    // Pong,
    Error(String),
    ChatHistory(VecDeque<ChatMessage>),
    ChatMessage(ChatMessage),
    SetUsers(Vec<PublicUser>),
    SetRoomConfig {
        name: String,
        public: bool,
        owner: PublicUser,
    },
    SetProblems(Option<Vec<ProblemPublic>>),
    SetTestResponse(TestResponse),
}

#[derive(Serialize, Debug)]
#[serde(tag = "t", content = "c")]
pub enum TestResponse {
    Error {
        message: String,
    },
    Ran {
        #[serde(rename = "failedTests")]
        failed_tests: Vec<FailedTestCase>,
        #[serde(rename = "okayTests")]
        okay_tests: Vec<TestCase>,
    },
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
    UserChat { author: PublicUser, content: String },
    Connection { username: String },
    Disconnection { username: String },
    RoundBegin,
    RoundEnd,
    Bad,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ConnId(pub String);
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct UserId(pub String);

const CHAT_MAX_MESSAGES: usize = 250;

pub struct Room {
    pub commands: Sender<RoomCommands>,
    commands_rx: Receiver<RoomCommands>,
    deletion_timer: Option<JoinHandle<()>>,
    config: RoomConfig,
    connections: HashMap<ConnId, (Sender<ConnectionCommands>, UserId)>,
    users: HashMap<UserId, (ConnId, User)>,
    // user id -> code
    editor_contents: HashMap<UserId, String>,
    chat_messages: VecDeque<ChatMessage>,
    problems: Vec<Problem>,
    round_in_progress: bool,
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
            editor_contents: Default::default(),
            problems: vec![
                Problem {
                    id: ObjectId::new(),
                    title: "Two Sum".into(),
                    description: "You know what this means".into(),
                    boilerplate_code: Code {
                        javascript: "function solve(nums, target) {\n  \n}\n".into(),
                        python: "def solve(nums, target):\n  pass\n".into(),
                    },
                    test_cases: vec![
                        TestCase {
                            input: r#"[[2, 7, 11, 15], 9]"#.into(),
                            output: "[0, 1]".into(),
                        },
                        TestCase {
                            input: r#"[[3, 2, 4], 6]"#.into(),
                            output: "[1, 2]".into(),
                        },
                    ],
                },
                Problem {
                    id: ObjectId::new(),
                    title: "Roman to Integer".into(),
                    description: "Convert roman numerals to integers".into(),
                    boilerplate_code: Code {
                        javascript: "function solve(romanNumeral) {\n  \n}\n".into(),
                        python: "def solve(roman_numeral):\n  pass\n".into(),
                    },
                    test_cases: vec![
                        TestCase {
                            input: r#""MCMXCIV""#.into(),
                            output: "1994".into(),
                        },
                        TestCase {
                            input: r#"LVIII"#.into(),
                            output: "58".into(),
                        },
                    ],
                },
            ],
            round_in_progress: false,
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
                self.send_connection(
                    &id,
                    &ServerSentCommand::SetRoomConfig {
                        name: self.config.name.clone(),
                        public: self.config.public,
                        owner: PublicUser {
                            id: self.config.owner.id.to_string(),
                            name: self.config.owner.name.clone(),
                            image: self.config.owner.image.clone(),
                        },
                    },
                )
                .await?;
                self.send_chat_message(ChatMessage::Connection { username })
                    .await?;
                self.send_all_command(&ServerSentCommand::SetUsers(
                    self.users
                        .iter()
                        .map(|(_, (_, user))| PublicUser {
                            id: user.id.to_string(),
                            name: user.name.to_string(),
                            image: user.image.clone(),
                        })
                        .collect(),
                ))
                .await?;

                self.editor_contents.insert(user_id.clone(), String::new());

                if self.round_in_progress {
                    self.send_all_command(&ServerSentCommand::SetProblems(Some(
                        self.problems
                            .iter()
                            .map(|prob| ProblemPublic {
                                id: prob.id.to_string(),
                                description: prob.description.clone(),
                                title: prob.title.clone(),
                                boilerplate_code: prob.boilerplate_code.clone(),
                                default_test_cases: prob.test_cases[0..2].to_vec(),
                            })
                            .collect(),
                    )))
                    .await?;
                }
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
                self.editor_contents.remove(&user_id).ok_or_else(|| {
                    anyhow::anyhow!("Trying to remove a editor content from an nonexistent user.")
                })?;

                self.send_all_command(&ServerSentCommand::SetUsers(
                    self.users
                        .iter()
                        .map(|(_, (_, user))| PublicUser {
                            id: user.id.to_string(),
                            name: user.name.to_string(),
                            image: user.image.clone(),
                        })
                        .collect(),
                ))
                .await?;

                log::info!("Room {}: connection {} removed", self.config.name, id.0);

                if self.connections.len() == 0 {
                    self.prime_deletion();
                }
            }
            ClientSent(conn_id, data) => {
                let (user_cmd, user_id) = match self.connections.get(&conn_id) {
                    Some(u) => u,
                    None => return Ok(false),
                };
                let (_, user) = match self.users.get(&user_id) {
                    Some(u) => u,
                    None => return Ok(false),
                };

                match data {
                    ClientSentCommand::Ping => (),
                    ClientSentCommand::SendChatMessage { content } => {
                        self.send_chat_message(ChatMessage::UserChat {
                            author: PublicUser {
                                name: user.name.clone(),
                                id: conn_id.0,
                                image: user.image.clone(),
                            },
                            content,
                        })
                        .await?;
                    }
                    ClientSentCommand::BeginRound => {
                        if self.round_in_progress {
                            return Ok(false);
                        }
                        self.send_chat_message(ChatMessage::RoundBegin).await?;
                        self.send_all_command(&ServerSentCommand::SetProblems(Some(
                            self.problems
                                .iter()
                                .map(|prob| ProblemPublic {
                                    id: prob.id.to_string(),
                                    description: prob.description.clone(),
                                    title: prob.title.clone(),
                                    boilerplate_code: prob.boilerplate_code.clone(),
                                    default_test_cases: prob.test_cases[0..2].to_vec(),
                                })
                                .collect(),
                        )))
                        .await?;
                        self.round_in_progress = true;
                    }
                    ClientSentCommand::SetEditorContent { content } => {
                        log::info!("{}", content);
                        self.editor_contents.insert(user_id.clone(), content);
                    }
                    ClientSentCommand::TestCode {
                        test_cases,
                        language,
                    } => {
                        if !["javascript", "python"].contains(&language.as_str()) {
                            return Ok(false);
                        }

                        let code = match self.editor_contents.get(user_id) {
                            Some(d) => d,
                            None => return Ok(false),
                        };

                        match judge::judge(&language, &code, &test_cases).await {
                            Err(err) => {
                                self.send_connection(
                                    &conn_id,
                                    &ServerSentCommand::SetTestResponse(TestResponse::Error {
                                        message: err.to_string(),
                                    }),
                                )
                                .await?;
                            }
                            Ok(results) => {
                                self.send_connection(
                                    &conn_id,
                                    &ServerSentCommand::SetTestResponse(TestResponse::Ran {
                                        failed_tests: results.failed_tests,
                                        okay_tests: results.okay_tests,
                                    }),
                                )
                                .await?;
                            }
                        }
                    }
                    ClientSentCommand::SubmitCode => {
                        log::info!("{:?} submitted", user_id);
                    }
                }
            }
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
