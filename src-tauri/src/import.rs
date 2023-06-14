use std::{cell::RefCell, collections::HashMap};

use rspc::Type;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{
    config::load_collection_config,
    error::FrontendError,
    model::{request_to_request_model, Collection, ImportWarning},
    tree::{GroupOptions, RequestTree, RequestTreeNode},
};
use http_rest_file::parser::Parser as RestFileParser;

pub mod postman;

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct ImportCollectionResult {
    pub collection: Collection,
    // @TODO: environment
    // @TODO: requestTree
    // @TODO: collectionConfig
    pub import_warnings: Vec<ImportWarning>,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct LoadRequestsResult {
    pub request_tree: RequestTree,
    pub errs: Vec<FrontendError>,
}

pub fn load_requests_for_collection(
    collection: &Collection,
) -> Result<LoadRequestsResult, FrontendError> {
    let mut parse_errs: Vec<FrontendError> = Vec::new();
    let mut nodes: HashMap<String, Vec<RefCell<RequestTreeNode>>> = HashMap::new();
    let mut root = RequestTreeNode::new_group(GroupOptions::FullPath(collection.path.clone()));

    for entry in WalkDir::new(&collection.path).into_iter().flatten() {
        // handle root node separately
        if entry.path().to_string_lossy() == collection.path {
            continue;
        }
        // @TODO: error
        let parent_path = entry.path().parent().unwrap();
        if entry.file_type().is_file() {
            if RestFileParser::has_valid_extension(&entry.file_name().to_string_lossy().to_string())
            {
                match RestFileParser::parse_file(entry.path()) {
                    Ok(mut model) => {
                        let path = entry.path().to_string_lossy().to_string();

                        let node = if model.requests.len() == 1 {
                            RequestTreeNode::new_request_node(
                                request_to_request_model(model.requests.remove(0), path.clone()),
                                path.clone(),
                            )
                        } else {
                            let mut file_group_node = RequestTreeNode::new_file_group(path.clone());
                            let request_nodes = model.requests.into_iter().map(|request| {
                                let request_model = request_to_request_model(request, path.clone());
                                RequestTreeNode::new_request_node(request_model, path.clone())
                            });
                            file_group_node.children.extend(request_nodes);
                            file_group_node
                        };
                        let entry = nodes.entry(parent_path.to_string_lossy().to_string()); //insert_(parent_path, RefCell::new(node));
                        let elements = entry.or_insert(Vec::new());
                        elements.push(RefCell::new(node));
                    }
                    Err(err) => parse_errs.push(err.into()),
                }
            }
        } else {
            // @TODO handle error
            let path = entry.path().to_string_lossy().to_string();
            let group_node = RequestTreeNode::new_group(GroupOptions::FullPath(path));
            let entry = nodes.entry(parent_path.to_string_lossy().to_string()); //insert_(parent_path, RefCell::new(node));
            let elements = entry.or_insert(Vec::new());
            elements.push(RefCell::new(group_node));
        }
    }

    let collection_config =
        load_collection_config(&collection.get_config_file_path()).unwrap_or_default();
    println!("Collection config: {:?}", collection_config);

    let mut parents: Vec<&mut RequestTreeNode> = vec![&mut root];
    while !parents.is_empty() {
        let parent = parents.pop().unwrap();
        if let Some(children) = nodes.remove(&parent.filepath) {
            let mut children: Vec<RequestTreeNode> = children
                .into_iter()
                .map(|child| child.into_inner())
                .collect();
            children.sort_by(|first, second| {
                match (
                    collection_config.path_orders.get(&first.filepath),
                    collection_config.path_orders.get(&second.filepath),
                ) {
                    (Some(first_order), Some(second_order)) => first_order.cmp(second_order),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Less,
                }
            });

            parent.children = children;
            let new_parents: Vec<&mut RequestTreeNode> = parent.children.iter_mut().collect();
            parents.extend(new_parents);
        };
    }

    println!("REQUEST TREE ROOT: {:?}", root);

    Ok(LoadRequestsResult {
        request_tree: RequestTree { root },
        errs: parse_errs,
    })
}
