use std::{cell::RefCell, collections::HashMap, fs, path::PathBuf};

use rspc::Type;
use serde::{Deserialize, Serialize};
use walkdir::{DirEntry, WalkDir};

use crate::{
    config::{load_collection_config, save_collection_config, save_workspace},
    error::{DisplayErrorKind, FrontendError},
    model::{
        request_to_request_model, Collection, CollectionConfig, ImportWarning, RequestModel,
        Workspace,
    },
    tree::{GroupOptions, RequestTree, RequestTreeNode},
};
use http_rest_file::{
    model::{HttpRestFile, ParseError},
    parser::Parser as RestFileParser,
};

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

pub const RELYNX_IGNORE_FILE: &str = ".relynxignore";

fn hidden_relynx_folder(entry: &DirEntry) -> bool {
    let path = entry.path();
    if path.is_file() {
        return false;
    }
    let in_folder = fs::read_dir(path);
    if in_folder.is_err() {
        return false;
    }
    in_folder
        .unwrap()
        .flatten()
        .any(|item| item.file_name() == RELYNX_IGNORE_FILE)
}

pub fn load_file_model(
    request_file_path: &std::path::PathBuf,
) -> Result<Vec<RequestModel>, ParseError> {
    let mut http_rest_file: HttpRestFile = RestFileParser::parse_file(&request_file_path)?;
    Ok(http_rest_file
        .requests
        .into_iter()
        .map(|request| request_to_request_model(request, &request_file_path))
        .collect::<Vec<RequestModel>>())
}

pub fn load_requests_for_collection(
    collection: &Collection,
) -> Result<LoadRequestsResult, FrontendError> {
    let mut parse_errs: Vec<FrontendError> = Vec::new();
    let mut nodes: HashMap<PathBuf, Vec<RefCell<RequestTreeNode>>> = HashMap::new();
    let mut root = RequestTreeNode::new_group(GroupOptions::FullPath(collection.path.clone()));

    let walker = WalkDir::new(&collection.path).into_iter();

    for entry in walker.filter_entry(|e| !hidden_relynx_folder(e)).flatten() {
        // handle root node separately
        if entry.path().to_string_lossy() == collection.path.to_string_lossy() {
            continue;
        }
        // @TODO: error
        let parent_path = entry.path().parent().unwrap();
        if entry.file_type().is_file() {
            if RestFileParser::has_valid_extension(&entry.file_name().to_string_lossy().to_string())
            {
                match RestFileParser::parse_file(entry.path()) {
                    Ok(mut model) => {
                        let path = entry.path().to_owned();

                        let node = if model.requests.len() == 1 {
                            RequestTreeNode::new_request_node(
                                request_to_request_model(model.requests.remove(0), &path),
                                path.clone(),
                            )
                        } else {
                            let mut file_group_node = RequestTreeNode::new_file_group(path.clone());
                            let request_nodes = model.requests.into_iter().map(|request| {
                                let request_model = request_to_request_model(request, &path);
                                RequestTreeNode::new_request_node(request_model, path.clone())
                            });
                            file_group_node.children.extend(request_nodes);
                            file_group_node
                        };
                        let entry = nodes.entry(parent_path.to_owned()); //insert_(parent_path, RefCell::new(node));
                        let elements = entry.or_insert(Vec::new());
                        elements.push(RefCell::new(node));
                    }
                    Err(err) => {
                        println!("Single parse err: {:?}", err);
                        parse_errs.push(err.into())
                    }
                }
            }
        } else {
            // @TODO handle error
            let path = entry.path();
            let group_node = RequestTreeNode::new_group(GroupOptions::FullPath(path.to_owned()));
            let entry = nodes.entry(parent_path.to_owned());
            let elements = entry.or_insert(Vec::new());
            elements.push(RefCell::new(group_node));
        }
    }

    let collection_config =
        load_collection_config(&collection.get_config_file_path()).unwrap_or_default();

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

    println!("PARSE_ERROS {:?}", parse_errs);
    Ok(LoadRequestsResult {
        request_tree: RequestTree { root },
        errs: parse_errs,
    })
}

pub fn create_jetbrains_collection(
    path: PathBuf,
    collection_name: String,
) -> Result<Collection, ()> {
    let mut collection_config = CollectionConfig::default();
    collection_config.name = collection_name;
    let collection = Collection {
        name: collection_config.name.clone(),
        path: path.clone(),
        description: "".to_string(),
        current_env_name: "".to_string(),
        import_warnings: Vec::new(),
        path_exists: true,
    };

    if let Ok(_) = save_collection_config(&collection_config, &collection.get_config_file_path()) {
        return Ok(collection);
    }
    return Err(());
}

pub fn import_jetbrains_folder(
    mut workspace: Workspace,
    jetbrains_folder: PathBuf,
    collection_name: String,
) -> Result<Workspace, FrontendError> {
    let result = create_jetbrains_collection(jetbrains_folder, collection_name);
    if result.is_err() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::Generic,
            "Could not import collection".to_string(),
        ));
    }
    let collection = result.unwrap();
    workspace.collections.push(collection);
    let result = save_workspace(&workspace);
    if result.is_err() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::Generic,
            "Could not import collection completely".to_string(),
        ));
    }
    return Ok(workspace);
}
