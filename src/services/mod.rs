pub mod file;
pub mod watch;
pub mod terminal;

pub use file::FileService;
pub use watch::WatchService;
pub use terminal::{PtySession, Sessions};

#[cfg(test)]
mod tests {
    use super::file::FileService;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_list_files() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // 创建测试文件
        std::fs::write(base.join("test.md"), "content").unwrap();
        std::fs::write(base.join("ignore.txt"), "hidden").unwrap();
        std::fs::write(base.join(".gitignore"), "*.txt").unwrap();

        let service = FileService::new(base.to_path_buf());
        let files = service.list_files("").await.unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "test.md");
    }

    #[tokio::test]
    async fn test_get_content() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        std::fs::write(base.join("test.md"), "hello world").unwrap();

        let service = FileService::new(base.to_path_buf());
        let content = service.get_content("test.md").await.unwrap();

        assert_eq!(content, "hello world");
    }

    #[tokio::test]
    async fn test_tree() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        std::fs::create_dir_all(base.join("subdir")).unwrap();
        std::fs::write(base.join("root.md"), "root").unwrap();
        std::fs::write(base.join("subdir/nested.md"), "nested").unwrap();

        let service = FileService::new(base.to_path_buf());
        let tree = service.tree().await.unwrap();

        assert_eq!(tree.name, base.file_name().unwrap().to_str().unwrap());
        assert!(tree.children.is_some());
    }
}
