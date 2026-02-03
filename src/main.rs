mod config;
mod models;
mod services;
mod routes;
mod websocket;

use actix_web::{web, App, HttpServer, middleware};
use config::Config;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    // 加载配置
    let config_path = PathBuf::from("mdterm.toml");
    let config = Config::from_file(&config_path)
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to load config: {}, using defaults", e);
            Config::default()
        });

    tracing::info!("Loaded {} contexts", config.contexts.len());

    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!("Starting server at {}", bind_addr);

    let watchers: websocket::Watchers = Arc::new(Mutex::new(HashMap::new()));
    let sessions: services::Sessions = Arc::new(Mutex::new(HashMap::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(watchers.clone()))
            .app_data(web::Data::new(sessions.clone()))
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api")
                    .configure(routes::configure_health)
                    .configure(routes::configure_contexts)
                    .service(
                        web::scope("/{context}")
                            .configure(routes::configure_files)
                            .service(
                                web::scope("/ws")
                                    .configure(websocket::configure_notify)
                                    .configure(websocket::configure_terminal)
                            )
                    )
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}
