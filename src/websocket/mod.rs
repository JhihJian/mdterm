pub mod notify;
pub mod terminal;

pub use notify::configure as configure_notify;
pub use terminal::configure as configure_terminal;

pub type Watchers = std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, crate::services::WatchService>>>;
