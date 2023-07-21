use std::path::PathBuf;

use crate::model::{PathOrder, RequestModel, Uuid};
use crate::sanitize::{sanitize_filename_with_options, Options as SanitizeOptions};
use http_rest_file::model::{HttpRestFile, HttpRestFileExtension, Request};
use rspc::Type;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Type, Debug, Default)]
pub struct RequestTree {
    pub root: RequestTreeNode,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct RequestTreeNode {
    pub id: Uuid,
    pub name: String,
    pub request: Option<RequestModel>,
    pub children: Vec<RequestTreeNode>,
    pub filepath: PathBuf,
    pub is_file_group: bool,
}

pub const DEFAULT_OPTIONS: SanitizeOptions<'static> = SanitizeOptions {
    replacement: "_",
    windows: cfg!(windows),
    truncate: true,
};

pub enum GroupOptions {
    FullPath(PathBuf),
}

impl RequestTreeNode {
    pub fn new_request_node(request_model: RequestModel, path: PathBuf) -> Self {
        // @TODO check if name works for a file name
        RequestTreeNode {
            name: sanitize_filename_with_options(&request_model.name, DEFAULT_OPTIONS),
            filepath: path,
            request: Some(request_model),
            ..Default::default()
        }
    }
    pub fn new_group(options: GroupOptions) -> Self {
        let mut node = RequestTreeNode::default();
        match options {
            GroupOptions::FullPath(path) => {
                node.filepath = path.clone();
                let group_path = std::path::PathBuf::from(path.clone());
                node.name = match group_path.file_name() {
                    Some(file_name) => file_name.to_string_lossy().to_string(),
                    None => sanitize_filename_with_options(
                        "Group_".to_string() + &uuid::Uuid::new_v4().to_string(),
                        DEFAULT_OPTIONS,
                    ),
                };
            }
        }
        node
    }

    pub fn new_file_group(path: PathBuf) -> Self {
        let mut node = RequestTreeNode::default();
        node.name = match path.file_name() {
            Some(file_name) => file_name.to_string_lossy().to_string(),
            None => sanitize_filename_with_options(
                "Request_".to_string() + &uuid::Uuid::new_v4().to_string(),
                DEFAULT_OPTIONS,
            ),
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

    pub fn is_request_node(&self) -> bool {
        self.request.is_some()
    }

    pub fn is_file_group(&self) -> bool {
        self.is_file_group
    }

    pub fn get_file_name(&self) -> String {
        let path = PathBuf::from(&self.filepath);
        path.file_name().unwrap().to_string_lossy().to_string()
    }
}

impl Default for RequestTreeNode {
    fn default() -> Self {
        RequestTreeNode {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            request: None,
            children: Vec::new(),
            filepath: PathBuf::new(),
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

impl TryFrom<&mut RequestTreeNode> for HttpRestFile {
    type Error = ();
    fn try_from(value: &mut RequestTreeNode) -> Result<Self, Self::Error> {
        TryFrom::<&RequestTreeNode>::try_from(value)
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
            path: Box::new(value.filepath.clone()),
            requests,
            errs: vec![],
            extension: Some(HttpRestFileExtension::Http),
        })
    }
}

pub fn correct_children_paths(node: &mut RequestTreeNode, path_orders: &mut PathOrder) {
    let mut nodes: Vec<&mut RequestTreeNode> = vec![node];
    while let Some(current) = nodes.pop() {
        current
            .children
            .iter_mut()
            .enumerate()
            .for_each(|(index, child)| {
                if let Some(file_name) = child.filepath.file_name() {
                    let new_path = current.filepath.join(file_name);

                    if path_orders.get(&child.filepath).is_some() {
                        path_orders.insert(new_path.clone(), index as u32);
                    }
                    child.filepath = new_path.clone();
                    if child.request.is_some() {
                        child.request.as_mut().unwrap().rest_file_path = new_path;
                    }
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
