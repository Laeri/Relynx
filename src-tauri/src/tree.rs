use std::path::PathBuf;

use crate::model::{RequestModel, Uuid};
use crate::sanitize::{sanitize_filename_with_options, Options as SanitizeOptions};
use http_rest_file::model::{HttpRestFile, HttpRestFileExtension, Request};
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

    pub fn any_child_with_name(&self, name: &str) -> bool {
        self.children.iter().any(|child| child.name == name)
    }

    pub fn is_folder(&self) -> bool {
        self.request.is_none() && !self.is_file_group
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

impl TryFrom<RequestTreeNode> for HttpRestFile {
    type Error = ();
    fn try_from(value: RequestTreeNode) -> Result<Self, Self::Error> {
        TryFrom::<&RequestTreeNode>::try_from(&value)
    }
}

impl TryFrom<&RequestTreeNode> for HttpRestFile {
    type Error = ();
    fn try_from(value: &RequestTreeNode) -> Result<Self, Self::Error> {
        // here we have a group, cannot be converted into a rest file as it is a directory
        if !value.is_file_group && value.request.is_none() {
            return Err(());
        }

        let requests = if value.is_file_group {
            value
                .children
                .iter()
                .map(|child| child.request.as_ref().unwrap().into())
                .collect::<Vec<Request>>()
        } else if value.request.is_some() {
            vec![value.request.as_ref().unwrap().into()]
        } else {
            return Err(());
        };

        Ok(HttpRestFile {
            path: Box::new(std::path::PathBuf::from(value.filepath.clone())),
            requests,
            errs: vec![],
            extension: Some(HttpRestFileExtension::Http),
        })
    }
}

pub fn correct_children_paths(node: &mut RequestTreeNode) {
    let mut nodes: Vec<&mut RequestTreeNode> = vec![node];
    while let Some(current) = nodes.pop() {
        let current_path = PathBuf::from(&current.filepath);
        current.children.iter_mut().for_each(|child| {
            let child_path = PathBuf::from(&child.filepath);
            if let Some(file_name) = child_path.file_name() {
                child.filepath = current_path.join(file_name).to_string_lossy().to_string();
            }
        });
        nodes.extend(
            current
                .children
                .iter_mut()
                .collect::<Vec<&mut RequestTreeNode>>(),
        );
    }
}
