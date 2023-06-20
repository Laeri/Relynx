use serde::{Deserialize, Serialize};

use crate::config::{load_collection_config, save_collection_config};
use crate::error::{DisplayErrorKind, FrontendError};
use crate::model::Collection;
use crate::tree::{correct_children_paths, RequestTreeNode};
use http_rest_file::model::*;
use http_rest_file::Serializer;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct DragAndDropParams {
    collection: Collection,
    drag_node_parent: RequestTreeNode,
    drag_node: RequestTreeNode,
    drop_node: RequestTreeNode,
    drop_index: u32,
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct DragAndDropResult {
    // @TODO
    new_drop_node: RequestTreeNode,
    remove_drag_node_parent: bool,
}

#[tauri::command]
pub fn drag_and_drop(params: DragAndDropParams) -> Result<DragAndDropResult, rspc::Error> {
    dd_check_preconditions(&params).map_err(Into::<rspc::Error>::into)?;

    let DragAndDropParams {
        collection,
        mut drag_node_parent,
        mut drag_node,
        mut drop_node,
        drop_index,
    } = params;

    // remove node from parent, this only used for persting the new parent here, in the frontend we
    // will also do this manually
    let in_parent_pos = drag_node_parent
        .children
        .iter()
        .position(|child| child.id == drag_node.id)
        .unwrap();
    drag_node_parent.children.remove(in_parent_pos);

    // Cases
    // Case 1a: We drag a folder -> into a folder === just move/rename the folder
    // Case 1b: We drag a single file (request or file_group) -> into a folder === just do a move/rename
    // Case 2a: We drag a request out of a file group -> into a file group (need to save both the
    // old and new request file)
    // Case 2b: we drag a request out of a file group -> into a folder === need to save old and new file
    // Case 2c: we drag a request -> into a file group === we need to save both the old and new

    let new_path = if drag_node_parent.is_folder() && drop_node.is_folder() {
        // this is case 1a/1b
        // just mv and be done
        let new_path = PathBuf::from(&drop_node.filepath).join(drag_node.get_file_name());
        // @TODO: err
        std::fs::rename(&drag_node.filepath, &new_path).map_err(|_err| {
            let msg = format!(
                "Could not move file or folder to target location. From: '{}', to: '{}'",
                &drag_node.filepath,
                &new_path.to_string_lossy().to_string()
            );
            Into::<rspc::Error>::into(FrontendError::new_with_message(
                DisplayErrorKind::DragAndDropError,
                msg,
            ))
        })?;
        new_path
    } else {
        // Case 2a,2b,2c
        // either we drag from a file group or we drag into a file group
        // create at new location
        let new_path = dd_create_new_location(&mut drag_node, &mut drop_node)
            .map_err(Into::<rspc::Error>::into)?;
        // Remove old file
        // If the parents drag node was a file group we need to remove the file from the group and
        // resave the group. If it was a single request we need to remove the old file
        dd_remove_old_location(&drag_node, &mut drag_node_parent)
            .map_err(Into::<rspc::Error>::into)?;
        new_path
    };


    // we dragged a folder/file within a folder, we only have to store the updated pathordering
    // @TODO: load CollectionConfig, update orderings for all paths within parent_node
    let config_file_path = collection.get_config_file_path();
    let mut collection_config = load_collection_config(&config_file_path).unwrap_or_default();
    let path_orders = &mut collection_config.path_orders;
    path_orders.remove(&drag_node.filepath);

    // update paths
    if drop_node.is_file_group {
        drag_node.filepath = drop_node.filepath.clone();
        drag_node
            .children
            .iter_mut()
            .for_each(|child| child.filepath = drop_node.filepath.clone());
    } else {
        drag_node.filepath = new_path.to_string_lossy().to_string();
        if let Some(request) = drag_node.request.as_mut() {
            request.rest_file_path = drag_node.filepath.clone();
        }
    }

    if drop_index >= drop_node.children.len() as u32 {
        drop_node.children.push(drag_node);
    } else {
        drop_node.children.insert(drop_index as usize, drag_node);
    }


    // need also to update path orders when changing paths of children
    let config_file_path = collection.get_config_file_path();
    let mut collection_config = load_collection_config(&config_file_path).unwrap_or_default();
    let path_orders = &mut collection_config.path_orders;

    correct_children_paths(&mut drop_node, path_orders);

    let _ = save_collection_config(&collection_config, &config_file_path);


    // if we removed the last child from the drag node then we need to remove its file as well
    let remove_drag_node_parent =
        drag_node_parent.is_file_group && drag_node_parent.children.is_empty();
    let result = DragAndDropResult {
        new_drop_node: drop_node,
        remove_drag_node_parent,
    };
    Ok(result)
}

fn dd_check_preconditions(params: &DragAndDropParams) -> Result<(), FrontendError> {
    let DragAndDropParams {
        collection,
        drag_node,
        drop_node,
        ..
    } = params;

    if collection.path.is_empty() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::DragAndDropError,
            "Invalid collection given, collection has no path",
        ));
    }

    if !drag_node.filepath.starts_with(&collection.path) {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::DragAndDropError,
            "The drag node is not within the given collection",
        ));
    }

    if !drop_node.filepath.starts_with(&collection.path) {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::DragAndDropError,
            "The drop node is not within the given collection",
        ));
    }

    if drag_node.filepath == drop_node.filepath {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::DragAndDropError,
            "Cannot drag node unto itself",
        ));
    }

    if drop_node.request.is_some() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::DragAndDropError,
            "Can only drop node into a group and not into a request",
        ));
    }

    // cannot drag a regular group into a file group, only requests can be put into file groups
    if drop_node.is_file_group && drag_node.request.is_none() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::DragAndDropError,
            "Cannot drop a group node into a file group.",
        ));
    }

    if drop_node.any_child_with_name(&drag_node.name) {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::DragAndDropError,
            "There exists already a node within the same name in the new parent. Rename the node first before dragging it."
        ));
    }

    if !PathBuf::from(&drop_node.filepath).exists() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::DragAndDropError,
            "The drop target node does not exist or has been removed".to_string(),
        ));
    }

    if !PathBuf::from(&drag_node.filepath).exists() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::DragAndDropError,
            "The drag node does not exist or has been removed".to_string(),
        ));
    }

    Ok(())
}

fn dd_create_new_location(
    drag_node: &mut RequestTreeNode,
    drop_node: &mut RequestTreeNode,
) -> Result<PathBuf, FrontendError> {
    // Create new file
    // If we drop into a file group the parent (aka the file group node) has to be resaved with the
    // new requests
    let request_file = if drop_node.is_file_group {
        // we drop a request into a file that already has requests, therefore save new file with
        // new request and remove old file
        let request_file: Result<HttpRestFile, ()> = drop_node.try_into();
        if request_file.is_err() {
            return Err(FrontendError::new_with_message(
                DisplayErrorKind::DragAndDropError,
                "Cannot convert request into a request file",
            ));
        }
        request_file.unwrap()
    } else {
        // otherwise we have moved a request into a folder and just need to save a new request
        // file
        let request_file: Result<HttpRestFile, ()> = drag_node.try_into();

        if request_file.is_err() {
            return Err(FrontendError::new_with_message(
                DisplayErrorKind::DragAndDropError,
                "Cannot convert request into a request file",
            ));
        }
        let new_path = PathBuf::from(&drop_node.filepath)
            .join(drag_node.request.as_ref().unwrap().get_request_file_name());
        // we need to take the new path the drag node would have after dragging
        let mut request_file = request_file.unwrap();
        request_file.path = Box::new(new_path);

        request_file
    };
    // @TODO log error
    Serializer::serialize_to_file(&request_file).map_err(|_err| {
        // @TODO: log serialize error
        FrontendError::new_with_message(
            DisplayErrorKind::DragAndDropError,
            "Could not write request to file. The request may be malformed.",
        )
    })?;
    Ok(*request_file.path)
}

fn dd_remove_old_location(
    drag_node: &RequestTreeNode,
    drag_node_parent: &mut RequestTreeNode,
) -> Result<(), FrontendError> {
    let drag_node_pos = drag_node_parent
        .children
        .iter()
        .position(|child| child.id == drag_node.id);
    if let Some(pos) = drag_node_pos {
        drag_node_parent.children.remove(pos);
    }

    // @TODO: @ERROR
    // if we dragged out of a file node and there are no requests left then remove the file
    if drag_node_parent.is_file_group {
        // if we drag out of a file group and it is empty remove it
        if drag_node_parent.children.is_empty() {
            std::fs::remove_file(&drag_node_parent.filepath).map_err(|_err| {
                FrontendError::new_with_message(
                    DisplayErrorKind::DragAndDropError,
                    "Could not remove old request file for drag node",
                )
            })?;
            return Ok(());
        }

        // otherwise we have to resave the file group but with the request removed from it

        // @TODO this might only be a warning?
        let new_drag_node_file: HttpRestFile = (drag_node_parent).try_into().map_err(|_err| {
            FrontendError::new_with_message(
                DisplayErrorKind::DragAndDropError,
                "Could not remove drag node from old parent.",
            )
        })?;
        // @TODO log error
        Serializer::serialize_to_file(&new_drag_node_file).map_err(|_err| {
            FrontendError::new_with_message(
                DisplayErrorKind::DragAndDropError,
                "Could not serialize to file",
            )
        })?;
        Ok(())
    } else {
        std::fs::remove_file(&drag_node.filepath).map_err(|_err| {
            FrontendError::new_with_message(
                DisplayErrorKind::DragAndDropError,
                "Could not remove old request file for drag node",
            )
        })?;
        Ok(())
        // we can just remove the old file
    }
}

#[derive(Serialize, Deserialize, rspc::Type, Debug)]
pub struct ReorderNodesParams {
    // @TODO
    collection: Collection,
    drag_node: RequestTreeNode,
    drop_node: RequestTreeNode,
    // needs u32 type as rspc cannot export larger types as there is an issue with json parsing for
    // large numbers
    drop_index: u32,
}

#[tauri::command]
pub fn reorder_nodes_within_parent(
    params: ReorderNodesParams,
) -> Result<RequestTreeNode, rspc::Error> {
    let ReorderNodesParams {
        collection,
        drag_node,
        mut drop_node,
        drop_index,
    } = params;

    let mut drop_index = drop_index as usize;

    let drop_node_path = PathBuf::from(&drop_node.filepath);
    if !drop_node_path.exists() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::ReorderError,
            "Reorder parent does not exist anymore. The file/folder might have been removed."
                .to_string(),
        )
        .into());
    }

    let position = drop_node
        .children
        .iter()
        .position(|child| child.id == drag_node.id);
    if position.is_none() {
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::ReorderError,
            "Could not reorder items".to_string(),
        )
        .into());
    }
    let position = position.unwrap();
    if drop_index > position {
        drop_index -= 1;
    }
    drop_node.children.remove(position);
    drop_node.children.insert(drop_index, drag_node);

    // if a request is dragged within a file group its ordering within changed and we have to
    // resave the file
    if drop_node.is_file_group {
        let rest_file: HttpRestFile = (&drop_node).try_into().unwrap();
        Serializer::serialize_to_file(&rest_file).map_err(|_err| {
            FrontendError::new_with_message(
                DisplayErrorKind::ReorderError,
                "Could not persist updated requests".to_string(),
            )
        })?;
    } else {
        // we dragged a folder/file within a folder, we only have to store the updated pathordering
        // @TODO: load CollectionConfig, update orderings for all paths within parent_node
        let config_file_path = collection.get_config_file_path();
        let mut collection_config = load_collection_config(&config_file_path).unwrap_or_default();
        let path_orders = &mut collection_config.path_orders;
        drop_node
            .children
            .iter()
            .enumerate()
            .for_each(|(index, child)| {
                path_orders.insert(child.filepath.clone(), index as u32);
            });
        save_collection_config(&collection_config, &config_file_path)?;
    }

    Ok(drop_node)
}
