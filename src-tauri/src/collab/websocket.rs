//! WebSocket Server for Collaborative Editing

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::Utf8Bytes;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use crate::collab::ot::TextOperation;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CollabMessage {
    Join { session_id: String, user_id: String, user_name: String },
    Leave { user_id: String },
    Operation { operation: TextOperation, client_version: u64 },
    Cursor { user_id: String, position: CursorPosition },
    Ack { version: u64 },
    Sync { content: String, version: u64 },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub line: u32,
    pub column: u32,
}

pub struct CollabClient {
    pub user_id: String,
    pub user_name: String,
    pub sender: mpsc::UnboundedSender<CollabMessage>,
}

pub struct CollabSession {
    pub id: String,
    pub document_id: String,
    pub clients: Arc<RwLock<HashMap<String, CollabClient>>>,
    pub operations: Arc<RwLock<Vec<TextOperation>>>,
    pub version: Arc<RwLock<u64>>,
}

impl CollabSession {
    pub fn new(id: String, document_id: String) -> Self {
        Self {
            id,
            document_id,
            clients: Arc::new(RwLock::new(HashMap::new())),
            operations: Arc::new(RwLock::new(Vec::new())),
            version: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn broadcast(&self, message: CollabMessage, exclude_user: Option<&str>) {
        let clients = self.clients.read().await;
        for (user_id, client) in clients.iter() {
            if exclude_user.map(|ex| ex == user_id).unwrap_or(false) {
                continue;
            }
            let _ = client.sender.send(message.clone());
        }
    }

    pub async fn add_client(&self, client: CollabClient) {
        let mut clients = self.clients.write().await;
        clients.insert(client.user_id.clone(), client);
    }

    pub async fn remove_client(&self, user_id: &str) {
        let mut clients = self.clients.write().await;
        clients.remove(user_id);
    }

    pub async fn apply_operation(&self, operation: TextOperation) -> Result<u64, String> {
        let mut operations = self.operations.write().await;
        let mut version = self.version.write().await;
        
        operations.push(operation);
        *version += 1;
        
        Ok(*version)
    }
}

pub struct WebSocketServer {
    sessions: Arc<RwLock<HashMap<String, Arc<CollabSession>>>>,
}

impl WebSocketServer {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await?;
        log::info!("WebSocket server listening on: {}", addr);

        while let Ok((stream, peer)) = listener.accept().await {
            let sessions = self.sessions.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, peer, sessions).await {
                    log::error!("WebSocket connection error: {}", e);
                }
            });
        }

        Ok(())
    }

    pub async fn create_session(&self,
        session_id: String,
        document_id: String,
    ) -> Arc<CollabSession> {
        let mut sessions = self.sessions.write().await;
        let session = Arc::new(CollabSession::new(session_id.clone(), document_id));
        sessions.insert(session_id, session.clone());
        session
    }
}

async fn handle_connection(
    stream: TcpStream,
    peer: SocketAddr,
    sessions: Arc<RwLock<HashMap<String, Arc<CollabSession>>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    let (tx, mut rx) = mpsc::unbounded_channel::<CollabMessage>();

    // Spawn task to send messages to WebSocket
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if ws_sender.send(Message::Text(Utf8Bytes::from(json))).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = ws_receiver.next().await {
        match msg? {
            Message::Text(text) => {
                match serde_json::from_str::<CollabMessage>(&text) {
                    Ok(collab_msg) => {
                        handle_collab_message(collab_msg, "user_id", &sessions, &tx).await;
                    }
                    Err(e) => {
                        log::error!("Failed to parse message: {}", e);
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    log::info!("WebSocket connection closed: {}", peer);
    Ok(())
}

async fn handle_collab_message(
    msg: CollabMessage,
    user_id: &str,
    sessions: &Arc<RwLock<HashMap<String, Arc<CollabSession>>>>,
    sender: &mpsc::UnboundedSender<CollabMessage>,
) {
    match msg {
        CollabMessage::Join { session_id, user_id, user_name } => {
            let sessions = sessions.read().await;
            if let Some(session) = sessions.get(&session_id) {
                let client = CollabClient {
                    user_id: user_id.clone(),
                    user_name,
                    sender: sender.clone(),
                };
                session.add_client(client).await;
                
                // Send current document state
                let version = *session.version.read().await;
                let ack = CollabMessage::Ack { version };
                let _ = sender.send(ack);
            }
        }
        CollabMessage::Operation {  .. } => {
            // Broadcast operation to all clients
            // TODO: Get session from context
        }
        CollabMessage::Cursor { user_id, position } => {
            // Broadcast cursor position
        }
        _ => {}
    }
}
