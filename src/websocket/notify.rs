use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::Message;
use crate::config::Config;
use crate::services::WatchService;
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

type Watchers = Arc<Mutex<HashMap<String, WatchService>>>;

pub async fn notify_handler(
    req: HttpRequest,
    stream: web::Payload,
    context: web::Path<String>,
    config: web::Data<Config>,
    watchers: web::Data<Watchers>,
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

    // 获取或创建 watcher
    let mut watchers_guard = watchers.lock().await;
    let watcher = if !watchers_guard.contains_key(&context_name) {
        let w = WatchService::new(context_config.path.clone());
        w.start().await.unwrap();
        watchers_guard.insert(context_name.clone(), w);
        watchers_guard.get(&context_name).unwrap().clone()
    } else {
        watchers_guard.get(&context_name).unwrap().clone()
    };
    drop(watchers_guard);

    let mut rx = watcher.subscribe();
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    // 启动任务发送事件
    actix_web::rt::spawn(async move {
        loop {
            tokio::select! {
                // 处理客户端消息
                result = msg_stream.next() => {
                    match result {
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
                // 发送文件事件
                result = rx.recv() => {
                    match result {
                        Ok(event) => {
                            let json = serde_json::to_string(&event).unwrap();
                            if session.text(json).await.is_err() {
                                break;
                            }
                        }
                        Err(_) => {
                            // Channel closed
                            break;
                        }
                    }
                }
            }
        }
        let _ = session.close(None).await;
    });

    Ok(response)
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/notify", web::get().to(notify_handler));
}
