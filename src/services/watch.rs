use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;

use crate::models::FileEvent;

pub struct WatchService {
    base_path: PathBuf,
    tx: broadcast::Sender<FileEvent>,
    _watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
}

impl Clone for WatchService {
    fn clone(&self) -> Self {
        Self {
            base_path: self.base_path.clone(),
            tx: self.tx.clone(),
            _watcher: Arc::clone(&self._watcher),
        }
    }
}

impl WatchService {
    pub fn new(base_path: PathBuf) -> Self {
        let (tx, _) = broadcast::channel(100);

        Self {
            base_path,
            tx,
            _watcher: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let tx = self.tx.clone();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                if let Some(path) = event.paths.first() {
                    Self::handle_event(path, &event.kind, &tx);
                }
            }
        })?;

        watcher.watch(&self.base_path, RecursiveMode::Recursive)?;

        let mut guard = self._watcher.lock().await;
        *guard = Some(watcher);

        Ok(())
    }

    fn handle_event(path: &Path, kind: &notify::EventKind, tx: &broadcast::Sender<FileEvent>) {
        // 只处理 .md 文件
        if !path.extension().and_then(|e| e.to_str()).map_or(false, |e| {
            e.eq_ignore_ascii_case("md")
        }) {
            return;
        }

        // 过滤临时文件
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name.ends_with("~") || file_name.contains(".swp") {
            return;
        }

        let mtime = std::fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let event = match kind {
            notify::EventKind::Create(_) => FileEvent::Created {
                path: path.to_string_lossy().to_string(),
                mtime,
            },
            notify::EventKind::Modify(_) => FileEvent::Modified {
                path: path.to_string_lossy().to_string(),
                mtime,
            },
            notify::EventKind::Remove(_) => FileEvent::Deleted {
                path: path.to_string_lossy().to_string(),
            },
            _ => return,
        };

        let _ = tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<FileEvent> {
        self.tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_watch_created() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path().to_path_buf();

        let service = WatchService::new(base.clone());
        let mut rx = service.subscribe();

        // 启动监听
        service.start().await.unwrap();

        // 创建文件
        std::fs::write(base.join("test.md"), "content").unwrap();

        // 等待事件
        let event = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("No event received");

        match event {
            FileEvent::Created { path, .. } => {
                assert!(path.contains("test.md"));
            }
            _ => panic!("Expected Created event, got {:?}", event),
        }
    }

    #[tokio::test]
    async fn test_watch_modified() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path().to_path_buf();

        let service = WatchService::new(base.clone());
        let mut rx = service.subscribe();

        service.start().await.unwrap();

        let test_file = base.join("test.md");
        std::fs::write(&test_file, "initial content").unwrap();

        // Drain the initial created event
        let _ = tokio::time::timeout(Duration::from_secs(2), rx.recv()).await;

        // Wait a bit before modifying
        sleep(Duration::from_millis(200)).await;

        // Modify the file
        std::fs::write(&test_file, "modified content").unwrap();

        // Wait for modify event
        let event = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("No event received");

        match event {
            FileEvent::Modified { path, .. } => {
                assert!(path.contains("test.md"));
            }
            _ => panic!("Expected Modified event, got {:?}", event),
        }
    }

    #[tokio::test]
    async fn test_watch_deleted() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path().to_path_buf();

        let service = WatchService::new(base.clone());
        let mut rx = service.subscribe();

        service.start().await.unwrap();

        let test_file = base.join("test.md");
        std::fs::write(&test_file, "content").unwrap();

        // Drain the initial created event
        let _ = tokio::time::timeout(Duration::from_secs(2), rx.recv()).await;

        // Wait a bit before deleting
        sleep(Duration::from_millis(200)).await;

        // Delete the file
        std::fs::remove_file(&test_file).unwrap();

        // Keep receiving events until we get a Deleted event or timeout
        let start = std::time::Instant::now();
        loop {
            let timeout_duration = Duration::from_secs(5).saturating_sub(start.elapsed());
            if timeout_duration.is_zero() {
                panic!("Timeout waiting for Deleted event");
            }

            match tokio::time::timeout(timeout_duration, rx.recv()).await {
                Ok(Ok(event)) => {
                    match &event {
                        FileEvent::Deleted { path } if path.contains("test.md") => {
                            // Got the expected event
                            return;
                        }
                        _ => continue, // Skip other events
                    }
                }
                _ => panic!("Timeout or error waiting for Deleted event"),
            }
        }
    }

    #[tokio::test]
    async fn test_watch_filters_non_md_files() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path().to_path_buf();

        let service = WatchService::new(base.clone());
        let mut rx = service.subscribe();

        service.start().await.unwrap();

        // Create a non-md file
        std::fs::write(base.join("test.txt"), "content").unwrap();

        // Wait a bit - should not receive any event
        let result = tokio::time::timeout(Duration::from_millis(500), rx.recv()).await;

        assert!(result.is_err(), "Should not receive events for non-md files");
    }

    #[tokio::test]
    async fn test_watch_filters_temp_files() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path().to_path_buf();

        let service = WatchService::new(base.clone());
        let mut rx = service.subscribe();

        service.start().await.unwrap();

        // Create temp files that should be filtered
        std::fs::write(base.join("test.md~"), "content").unwrap();
        std::fs::write(base.join(".test.md.swp"), "content").unwrap();

        // Wait a bit - should not receive any event
        let result = tokio::time::timeout(Duration::from_millis(500), rx.recv()).await;

        assert!(result.is_err(), "Should not receive events for temp files");
    }
}
