use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FileEvent {
    #[serde(rename = "created")]
    Created { path: String, mtime: i64 },
    #[serde(rename = "modified")]
    Modified { path: String, mtime: i64 },
    #[serde(rename = "deleted")]
    Deleted { path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalHandshake {
    pub session_id: String,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum TerminalClientMessage {
    #[serde(rename = "input")]
    Input { data: String },
    #[serde(rename = "resize")]
    Resize { cols: u16, rows: u16 },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum TerminalServerMessage {
    #[serde(rename = "handshake")]
    Handshake(TerminalHandshake),
    #[serde(rename = "output")]
    Output { data: String },
    #[serde(rename = "exit")]
    Exit { code: i32 },
}
