mod config;

use config::Config;
use std::path::PathBuf;

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

    // 启动服务器 (占位)
    tracing::info!("Server will start at {}:{}", config.server.host, config.server.port);

    Ok(())
}
