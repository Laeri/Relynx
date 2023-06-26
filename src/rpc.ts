import { createClient } from '@rspc/client'
import { TauriTransport } from '@rspc/tauri'
import { Procedures, Workspace, Collection, AddCollectionsResult, ImportCollectionResult, LoadRequestsResult, RunRequestCommand, RequestResult, RequestModel, SaveRequestCommand, RequestTreeNode, DragAndDropResult, Environment, ValidateGroupNameResult } from './bindings';
import { FError } from './common/errorhandling';
import { CancellationToken } from './model/error';

export const api = createClient<Procedures>({
  transport: new TauriTransport()
})

class Backend {

  constructor() {

  }

  loadWorkspace(): Promise<Workspace> {
    let result = api.query(['load_workspace']);
    return result;
  }

  removeCollection(collection: Collection): Promise<Workspace> {
    return api.query(['remove_collection', collection]);
  }

  selectDirectory(): Promise<string> {
    return api.query(['select_directory']);
  }

  selectFile(): Promise<string> {
    return api.query(['select_file']);
  }

  is_directory_empty(path: string): Promise<boolean> {
    return api.query(['is_directory_empty', path]);
  }

  updateWorkspace(workspace: Workspace): Promise<null> {
    return api.query(['update_workspace', workspace]);
  }

  addExistingCollections(path: string, workspace: Workspace): Promise<AddCollectionsResult> {
    let result = api.query(['add_existing_collections', { path, workspace }]);
    console.log("RESULT: ", result);
    return result;
  }

  loadRequestsForCollection(collection: Collection): Promise<LoadRequestsResult> {
    let result = api.query(['load_requests_for_collection', collection])
    return result;
  }

  loadEnvironments(collectionPath: string): Promise<Environment[]> {
    return api.query(['load_environments', collectionPath]);
  }

  saveEnvironments(collection: Collection, environments: Environment[]): Promise<null> {
    return api.query(['save_environments', { collection_path: collection.path, environments: environments }]);
  }

  importPostmanCollection(workspace: Workspace, import_postman_path: string, import_result_path: string): Promise<ImportCollectionResult> {
    let result = api.query(['import_postman_collection', { workspace, import_postman_path, import_result_path }]);
    return result;
  }

  importJetbrainsFolder(workspace: Workspace, import_jetbrains_folder: string, collection_name: string): Promise<Workspace> {
    let result = api.query(['import_jetbrains_folder', { workspace, import_jetbrains_folder, collection_name }]);
    return result;
  }

  runRequest(runRequestCommand: RunRequestCommand, cancellationToken: CancellationToken): Promise<RequestResult> {
    return new Promise((resolve, reject) => {
      if (cancellationToken.cancelled) {
        console.log('cancelled');
        return
      }
      api.query(['run_request', runRequestCommand]).then((result: any) => {
        if (cancellationToken.cancelled) {
          console.log('cancelled');
          return
        }
        console.log('resolved');
        resolve(result);
      }).catch((cancel_val: any) => {
        if (cancellationToken.cancelled) {
          console.log('cancelled');
          return
        }
        console.log('resolved')
        reject(cancel_val)
      });
    });
  }

  getResponseFilepath(request_path: string): Promise<string> {
    let result = api.query(['get_response_filepath', request_path]);
    return result;
  }

  validateResponseFilepath(filepath: string): Promise<boolean> {
    return api.query(['validate_response_filepath', filepath])
  }

  // @TODO: check if current request one parameter?
  // Promise result contains new path??? @TODO
  saveRequest(requests: RequestModel[], collection: Collection, requestName: string): Promise<string> {
    let command: SaveRequestCommand = { requests: requests, collection: collection, request_name: requestName };
    return api.query(['save_request', command]);
  }

  copyToClipboard(value: string): Promise<null> {
    return api.query(['copy_to_clipboard', value]);
  }

  openFolderNative(path: string): Promise<null> {
    return api.query(['open_folder_native', path]);
  }

  addRequestNode(collection: Collection, parent: RequestTreeNode, request_name: string, requestsInSameFile: RequestModel[]): Promise<RequestTreeNode> {
    let result = api.query(['add_request_node', { collection: collection, parent: parent, request_name: request_name, requests_in_same_file: requestsInSameFile }]);
    return result;
  }

  addGroupNode(collection: Collection, parent: RequestTreeNode, groupName: string): Promise<RequestTreeNode> {
    let result = api.query(['add_group_node', { collection: collection, parent: parent, group_name: groupName }]);
    return result;
  }

  validateGroupName(old_path: string, new_name: string): Promise<ValidateGroupNameResult> {
    return api.query(['validate_group_name', { old_path: old_path, new_name: new_name }]);
  }

  // returns new path
  renameGroup(collection_path: string, old_path: string, new_name: string): Promise<string> {
    return api.query(['rename_group', { collection_path: collection_path, old_path: old_path, new_name: new_name }]);
  }

  deleteNode(collection: Collection, node: RequestTreeNode, fileNode: RequestTreeNode | null): Promise<null> {
    return api.query(['delete_node', { collection: collection, node: node, file_node: fileNode }]);
  }

  dragAndDrop(collection: Collection, dragNodeParent: RequestTreeNode, dragNode: RequestTreeNode, dropNode: RequestTreeNode, dropIndex: number): Promise<DragAndDropResult> {
    let result = api.query(['drag_and_drop', { collection: collection, drag_node_parent: dragNodeParent, drag_node: dragNode, drop_node: dropNode, drop_index: dropIndex }]);
    return result;
  }

  // collection *Collection, dragNode *RequestTreeNode, dropNode *RequestTreeNode, dropIndex int
  reorderNodesWithinParent(collection: Collection, dragNode: RequestTreeNode, dropNode: RequestTreeNode, dropIndex: number): Promise<RequestTreeNode> {
    let result = api.query(['reorder_nodes_within_parent', { collection: collection, drag_node: dragNode, drop_node: dropNode, drop_index: dropIndex }]);
    return result;
  }

  // collection *Collection, dragNode *RequestTreeNode, dropNode *RequestTreeNode, dropIndex int
  hideGroup(path: string): Promise<null> {
    return api.query(['hide_group', path]);
  }

  logFrontendError(error: FError): Promise<void> {
    // @TODO: IMPLEMENT
    return Promise.resolve();
  }
}


export const backend = new Backend();


