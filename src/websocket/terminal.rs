use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::Message;
use crate::config::Config;
use crate::models::{TerminalClientMessage, TerminalServerMessage, TerminalHandshake};
use crate::services::{PtySession, Sessions};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures::stream::StreamExt;
use std::time::Duration;
use uuid::Uuid;

pub async fn terminal_handler(
    req: HttpRequest,
    stream: web::Payload,
    context: web::Path<String>,
    config: web::Data<Config>,
    sessions: web::Data<Sessions>,
) -> Result<HttpResponse, actix_web::Error> {
    let context_name = context.into_inner();
    let context_config = match config.get_context(&context_name) {
        Some(c) => c,
        None => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "code": "CONTEXT_NOT_FOUND",
                "message": format!("Context '{}' not found", context_name)
            })));
        }
    };

    let session_id = Uuid::new_v4().to_string();
    let pty_session = match PtySession::new(
        session_id.clone(),
        context_config.path.clone(),
        &context_config.command,
        &context_config.env,
    ) {
        Ok(s) => s,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "code": "PTY_ERROR",
                "message": e.to_string()
            })));
        }
    };

    // 发送握手消息
    let handshake = TerminalServerMessage::Handshake(TerminalHandshake {
        session_id: session_id.clone(),
        cols: 80,
        rows: 24,
    });

    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    // 发送握手
    let handshake_json = serde_json::to_string(&handshake).unwrap();
    if session.text(handshake_json).await.is_err() {
        return Ok(response);
    }

    // 注册会话
    sessions.lock().await.insert(session_id.clone(), pty_session);

    let session_id_clone = session_id.clone();
    let sessions_clone = sessions.clone();

    // 启动任务处理 PTY 和 WebSocket
    actix_web::rt::spawn(async move {
        let mut read_buf = [0u8; 8192];

        loop {
            tokio::select! {
                // 处理客户端消息
                result = msg_stream.next() => {
                    match result {
                        Some(Ok(Message::Text(text))) => {
                            if let Ok(msg) = serde_json::from_str::<TerminalClientMessage>(&text) {
                                let mut sessions = sessions_clone.lock().await;
                                if let Some(pty) = sessions.get_mut(&session_id_clone) {
                                    match msg {
                                        TerminalClientMessage::Input { data } => {
                                            if let Ok(bytes) = BASE64.decode(&data) {
                                                let _ = pty.write(&bytes);
                                            }
                                        }
                                        TerminalClientMessage::Resize { cols, rows } => {
                                            let _ = pty.resize(rows, cols);
                                        }
                                    }
                                }
                            }
                        }
                        Some(Ok(Message::Ping(msg))) => {
                            if session.pong(&msg).await.is_err() {
                                break;
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            break;
                        }
                        Some(Err(_)) | None => break,
                        _ => {}
                    }
                }
                // 读取 PTY 输出
                _ = tokio::time::sleep(Duration::from_millis(50)) => {
                    let mut sessions = sessions_clone.lock().await;
                    if let Some(pty) = sessions.get_mut(&session_id_clone) {
                        match pty.read(&mut read_buf) {
                            Ok(n) if n > 0 => {
                                let data = BASE64.encode(&read_buf[..n]);
                                let msg = TerminalServerMessage::Output { data };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    if session.text(json).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            Ok(_) => {}
                            Err(_) => {
                                // PTY closed
                                let _ = session.text(
                                    serde_json::to_string(&TerminalServerMessage::Exit { code: 0 }).unwrap()
                                ).await;
                                break;
                            }
                        }
                    }
                }
            }
        }

        // 清理会话
        sessions_clone.lock().await.remove(&session_id_clone);
        let _ = session.close(None).await;
    });

    Ok(response)
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/terminal", web::get().to(terminal_handler));
}
