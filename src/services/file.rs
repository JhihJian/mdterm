use crate::models::file::{FileInfo, TreeNode};
use ignore::gitignore::Gitignore;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use tokio::fs;
use futures::Future;

pub struct FileService {
    base_path: PathBuf,
    gitignore: Option<Gitignore>,
}

impl FileService {
    pub fn new(base_path: PathBuf) -> Self {
        let gitignore = Self::load_gitignore(&base_path);
        Self { base_path, gitignore }
    }

    fn load_gitignore(base_path: &Path) -> Option<Gitignore> {
        let gitignore_path = base_path.join(".gitignore");
        if gitignore_path.exists() {
            let (gi, _err) = Gitignore::new(gitignore_path);
            Some(gi)
        } else {
            None
        }
    }

    fn resolve_path(&self, relative: &str) -> PathBuf {
        self.base_path.join(relative).canonicalize().unwrap_or_else(|_| self.base_path.join(relative))
    }

    fn is_md_file(path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("md"))
            .unwrap_or(false)
    }

    fn contains_md_files(dir_path: &Path) -> bool {
        std::fs::read_dir(dir_path)
            .map(|mut entries| {
                entries.any(|entry| {
                    entry.ok().map(|e| {
                        let path = e.path();
                        path.is_file() && Self::is_md_file(&path)
                    }).unwrap_or(false)
                })
            })
            .unwrap_or(false)
    }

    pub async fn list_files(&self, relative_path: &str) -> Result<Vec<FileInfo>, std::io::Error> {
        let full_path = self.resolve_path(relative_path);

        let mut files = Vec::new();
        let mut entries = fs::read_dir(&full_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Skip hidden files/dirs
            if path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }

            let metadata = entry.metadata().await?;
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Filter .md files and check gitignore
            if path.is_file() {
                if !Self::is_md_file(&path) {
                    continue;
                }
                if let Some(gi) = &self.gitignore {
                    let relative = path.strip_prefix(&self.base_path).unwrap_or(&path);
                    if gi.matched(relative, path.is_dir()).is_ignore() {
                        continue;
                    }
                }
            }

            let relative_to_base = path.strip_prefix(&self.base_path)
                .ok()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string();

            let mtime = metadata.modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            files.push(FileInfo {
                name,
                path: relative_to_base,
                size: metadata.len(),
                mtime,
                is_dir: metadata.is_dir(),
            });
        }

        files.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(files)
    }

    pub async fn get_content(&self, relative_path: &str) -> Result<String, std::io::Error> {
        let full_path = self.resolve_path(relative_path);
        fs::read_to_string(full_path).await
    }

    pub async fn tree(&self) -> Result<TreeNode, std::io::Error> {
        let base_name = self.base_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let children = self.build_tree_recursive(&self.base_path).await?;
        Ok(TreeNode::dir(base_name, String::new(), children))
    }

    fn build_tree_recursive<'a>(
        &'a self,
        dir_path: &'a Path,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<TreeNode>, std::io::Error>> + Send + 'a>> {
        Box::pin(async move {
            let mut nodes = Vec::new();
            let mut entries = fs::read_dir(dir_path).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                // Skip hidden files/dirs
                if path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with('.'))
                    .unwrap_or(false)
                {
                    continue;
                }

                // Check gitignore
                if let Some(gi) = &self.gitignore {
                    let relative = path.strip_prefix(&self.base_path).unwrap_or(&path);
                    if gi.matched(relative, path.is_dir()).is_ignore() {
                        continue;
                    }
                }

                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                let relative_to_base = path.strip_prefix(&self.base_path)
                    .ok()
                    .and_then(|p| p.to_str())
                    .unwrap_or("")
                    .to_string();

                if path.is_file() {
                    if Self::is_md_file(&path) {
                        let metadata = entry.metadata().await?;
                        let mtime = metadata.modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);

                        nodes.push(TreeNode::file(name, relative_to_base, metadata.len(), mtime));
                    }
                } else if path.is_dir() {
                    // Only include directories that contain markdown files
                    if Self::contains_md_files(&path) {
                        let children = self.build_tree_recursive(&path).await?;
                        nodes.push(TreeNode::dir(name, relative_to_base, children));
                    }
                }
            }

            nodes.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(nodes)
        })
    }
}
