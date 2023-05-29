use crate::model::{RequestModel, Uuid};
use crate::sanitize::{sanitize_filename_with_options, Options as SanitizeOptions};
use rspc::Type;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct RequestTree {
    pub root: RequestTreeNode,
}

impl Default for RequestTree {
    fn default() -> Self {
        RequestTree {
            root: RequestTreeNode::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct RequestTreeNode {
    pub id: Uuid,
    pub name: String,
    pub request: Option<RequestModel>,
    pub children: Vec<RequestTreeNode>,
    pub filepath: String,
    pub is_file_group: bool,
}

pub const DEFAULT_OPTIONS: SanitizeOptions<'static> = SanitizeOptions {
    replacement: "_",
    windows: cfg!(windows),
    truncate: true,
};

pub enum GroupOptions {
    FullPath(String),
}

impl RequestTreeNode {
    pub fn new_request_node(request_model: RequestModel, path: String) -> Self {
        let mut node = RequestTreeNode::default();
        // @TODO check if name works for a file name

        node.name = sanitize_filename_with_options(&request_model.name, DEFAULT_OPTIONS);
        node.filepath = path;
        node.request = Some(request_model);
        node
    }
    pub fn new_group(options: GroupOptions) -> Self {
        let mut node = RequestTreeNode::default();
        match options {
            GroupOptions::FullPath(path) => {
                node.filepath = path.clone();
                let group_path = std::path::PathBuf::from(path.clone());
                node.name = match group_path.file_name() {
                    Some(file_name) => file_name.to_string_lossy().to_string(),
                    None => sanitize_filename_with_options(path, DEFAULT_OPTIONS),
                };
            }
        }
        node
    }

    pub fn new_file_group(path: String) -> Self {
        let mut node = RequestTreeNode::default();
        let file_path = std::path::PathBuf::from(path.clone());
        node.name = match file_path.file_name() {
            Some(file_name) => file_name.to_string_lossy().to_string(),
            None => sanitize_filename_with_options(path.clone(), DEFAULT_OPTIONS),
        };
        node.is_file_group = true;
        node.filepath = path;
        node
    }
}

impl Default for RequestTreeNode {
    fn default() -> Self {
        RequestTreeNode {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            request: None,
            children: Vec::new(),
            filepath: String::new(),
            is_file_group: false,
        }
    }
}
