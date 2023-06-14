import { createClient } from '@rspc/client'
import { TauriTransport } from '@rspc/tauri'
import { Procedures, Workspace, Collection, AddCollectionsResult, ImportCollectionResult, LoadRequestsResult, RunRequestCommand, RequestResult, RequestModel, SaveRequestCommand, RequestTreeNode, DragAndDropResult } from './bindings';
import { FError } from './common/errorhandling';

export const api = createClient<Procedures>({
  transport: new TauriTransport()
})

class Backend {

  constructor() {

  }

  loadWorkspace(): Promise<Workspace> {
    console.log('try load');
    let result = api.query(['load_workspace']);
    console.log('result: ', result);
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
    return api.query(['add_existing_collections', { path, workspace }]);
  }

  loadRequestsForCollection(collection: Collection): Promise<LoadRequestsResult> {
    let result = api.query(['load_requests_for_collection', collection])
    console.log('load requests for collection result: ', result);
    return result;
  }

  importPostmanCollection(workspace: Workspace, import_postman_path: string, import_result_path: string): Promise<ImportCollectionResult> {
    return api.query(['import_postman_collection', { workspace, import_postman_path, import_result_path }]);
  }

  runRequest(runRequestCommand: RunRequestCommand): Promise<RequestResult> {
    return api.query(['run_request', runRequestCommand]);
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
    console.log('RESULT: ', result);
    return result;
  }

  addGroupNode(collection: Collection, parent: RequestTreeNode, groupName: string): Promise<RequestTreeNode> {
    let result = api.query(['add_group_node', { collection: collection, parent: parent, group_name: groupName }]);
    console.log('RESULT: ', result);
    return result;
  }

  deleteNode(collection: Collection, node: RequestTreeNode, fileNode: RequestTreeNode | null): Promise<null> {
    return api.query(['delete_node', { collection: collection, node: node, file_node: fileNode }]);
  }

  dragAndDrop(collection: Collection, dragNodeParent: RequestTreeNode, dragNode: RequestTreeNode, dropNode: RequestTreeNode, dropIndex: number): Promise<DragAndDropResult> {
    console.log('drop index: ', dropIndex);
    let result = api.query(['drag_and_drop', { collection: collection, drag_node_parent: dragNodeParent, drag_node: dragNode, drop_node: dropNode, drop_index: dropIndex }]);

    var jsonPretty = JSON.stringify(JSON.parse(JSON.stringify(result)));
    console.log('RESULT DRAG AND DROP: ', jsonPretty)
    console.log('"drag_and_drop result: ', result);
    return result;
  }

  // collection *Collection, dragNode *RequestTreeNode, dropNode *RequestTreeNode, dropIndex int
  reorderNodesWithinParent(collection: Collection, dragNode: RequestTreeNode, dropNode: RequestTreeNode, dropIndex: number): Promise<RequestTreeNode> {
    let result = api.query(['reorder_nodes_within_parent', { collection: collection, drag_node: dragNode, drop_node: dropNode, drop_index: dropIndex }]);
    console.log('reorder nodes result: ', result);
    return result;
  }

  logFrontendError(error: FError): Promise<void> {
    // @TODO: IMPLEMENT
    return Promise.resolve();
  }
}


export const backend = new Backend();


