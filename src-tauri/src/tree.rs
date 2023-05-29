use crate::model::{RequestModel, Uuid};
use rspc::Type;
use serde::{Deserialize, Serialize};

/* Id           UUID               `json:"Id"`
   Name         string             `json:"Name"`
   RequestModel *RequestModel      `json:"RequestModel"`
   Children     []*RequestTreeNode `json:"Children"`
   // path to the 'folder'/'file' within the file system
   Filepath string `json:"Filepath"`
   // if this node is actually a file (a single file can contain multiple requests)
   IsFileGroup bool `json:"IsFileGroup"`
*/

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct RequestTree {
    pub root: RequestTreeNode,
}

impl Default for RequestTree {
    fn default() -> Self {
        RequestTree {
            root: RequestTreeNode::default()
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

impl RequestTreeNode {
    pub fn new_request_node(request_model: RequestModel, path: String) -> Self {
        let mut node = RequestTreeNode::default();
        node.filepath = path;
        node.request = Some(request_model);
        node
    }
    pub fn new_group(path: String) -> Self {
        let mut node = RequestTreeNode::default();
        node.filepath = path;
        node
    }

    pub fn new_file_group(path: String) -> Self {
        let mut node = RequestTreeNode::default();
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
