import { createClient } from '@rspc/client'
import { TauriTransport } from '@rspc/tauri'
import { Procedures, Workspace, Collection, AddCollectionsResult, ImportCollectionResult, LoadRequestsResult, RunRequestCommand, RequestResult, RequestModel, SaveRequestCommand, RequestTreeNode, DragAndDropResult, Environment, ValidateGroupNameResult, LicenseData } from './bindings';
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

  selectDirectory(onSelect: (directoryPath: string) => void) {
    api.query(['select_directory']).then((result: string | null) => {
      if (result) {
        onSelect(result);
      }
    });
  }

  selectFile(onSelect: (filepath: string) => void) {
    api.query(['select_file']).then((result: string | null) => {
      if (result) {
        onSelect(result);
      }
    });

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
    return api.query(['load_requests_for_collection', collection])
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
        return
      }
      api.query(['run_request', runRequestCommand]).then((result: RequestResult) => {
        if (cancellationToken.cancelled) {
          return
        }
        resolve(result);
      }).catch((cancel_val: any) => {
        if (cancellationToken.cancelled) {
          return
        }
        reject(cancel_val)
      });
    });
  }

  getResponseFilepath(request_path: string, onSelected: (request_path: string) => void) {
    api.query(['get_response_filepath', request_path]).then((result: string | null) => {
      if (result) {
        onSelected(result);
      }
    });
  }

  validateResponseFilepath(filepath: string): Promise<boolean> {
    return api.query(['validate_response_filepath', filepath])
  }

  // @TODO: check if current request one parameter?
  // Promise result contains new path??? @TODO
  saveRequest(requests: RequestModel[], collection: Collection, oldName: string): Promise<string> {
    let command: SaveRequestCommand = { requests: requests, collection: collection, old_name: oldName };
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

  chooseFileRelativeTo(base_path: string, onSelect: (path: string) => void) {
    api.query(['choose_file_relative_to', { base_path: base_path }]).then((result: string | null) => {
      if (result) {
        onSelect(result);
      }
    });
  }

  loadLicenseData(): Promise<LicenseData> {
    return api.query(['load_license_data']);
  }

  saveLicenseData(licenseData: LicenseData): Promise<null> {
    return api.mutation(['save_license_data', licenseData]);
  }

  isSignatureValid(licenseData: LicenseData): Promise<boolean> {
    return api.query(['is_signature_valid', licenseData]);
  }
}


export const backend = new Backend();


