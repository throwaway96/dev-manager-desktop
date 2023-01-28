use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock, Weak};

use russh::client::Msg;
use russh::Channel;
use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{Mutex as AsyncMutex, Semaphore};
use uuid::Uuid;
use vt100::Parser;

use connection::Connection;

use crate::session_manager::connection::ConnectionsMap;
use crate::session_manager::shell::ShellsMap;

mod connection;
mod device;
mod error;
mod handler;
mod manager;
mod proc;
mod shell;

#[derive(Default)]
pub struct SessionManager {
    lock: AsyncMutex<()>,
    pub(crate) shells: Arc<Mutex<ShellsMap>>,
    connections: Arc<Mutex<ConnectionsMap>>,
}

pub struct Proc {
    pub(crate) command: String,
    pub(crate) ch: AsyncMutex<Option<Channel<Msg>>>,
}

#[derive(Clone, Serialize)]
pub struct ProcData {
    pub index: u64,
    pub data: Vec<u8>,
}

pub struct Shell {
    pub token: ShellToken,
    connection: Weak<Connection>,
    pub(crate) channel: AsyncMutex<Option<Channel<Msg>>>,
    pub(crate) sender: AsyncMutex<Option<UnboundedSender<Vec<u8>>>>,
    pub(crate) parser: Mutex<Parser>,
    pub(crate) ready: Semaphore,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ShellToken {
    pub connection_id: Uuid,
    pub channel_id: String,
}

#[derive(Hash, Clone, Debug, Serialize)]
pub struct ShellData {
    pub token: ShellToken,
    pub fd: u32,
    pub data: Vec<u8>,
}

#[derive(Clone, Serialize, Debug)]
pub struct ShellBuffer {
    rows: Vec<Vec<u8>>,
    cursor: (u16, u16),
}

#[derive(Debug, Serialize, Clone)]
pub struct Error {
    pub message: String,
    #[serde(flatten)]
    pub kind: ErrorKind,
}

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum ErrorKind {
    Message,
    Unimplemented,
    NeedsReconnect,
    Authorization,
    NotFound,
    EmptyData,
    ExitStatus { status: u32, output: Vec<u8> },
}