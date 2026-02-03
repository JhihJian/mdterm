pub mod notify;

pub use notify::configure as configure_notify;

pub type Watchers = std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, crate::services::WatchService>>>;
