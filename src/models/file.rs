use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub mtime: i64,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtime: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<TreeNode>>,
}

impl TreeNode {
    pub fn file(name: String, path: String, size: u64, mtime: i64) -> Self {
        Self {
            name,
            path,
            size: Some(size),
            mtime: Some(mtime),
            children: None,
        }
    }

    pub fn dir(name: String, path: String, children: Vec<TreeNode>) -> Self {
        Self {
            name,
            path,
            size: None,
            mtime: None,
            children: Some(children),
        }
    }
}
