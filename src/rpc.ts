import { createClient } from '@rspc/client'
import { TauriTransport } from '@rspc/tauri'
import { Procedures, Workspace, Collection, AddCollectionsResult, ImportCollectionResult, LoadRequestsResult, RunRequestCommand, RequestResult, RequestModel, SaveRequestCommand } from './bindings';

export const api = createClient<Procedures>({
  transport: new TauriTransport()
})

class Backend {

  constructor() {

  }

  loadWorkspace(): Promise<Workspace> {
    return api.query(['load_workspace']);
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
    return api.query(['load_requests_for_collection', collection])
  }

  importPostmanCollection(workspace: Workspace, import_postman_path: string, import_result_path: string): Promise<ImportCollectionResult> {
    return api.query(['import_postman_collection', { workspace, import_postman_path, import_result_path }]);
  }

  runRequest(runRequestCommand: RunRequestCommand): Promise<RequestResult> {
    return api.query(['run_request', runRequestCommand]);
  }

  // @TODO: check if current request one parameter?
  saveRequest(requests: RequestModel[], collection: Collection, requestName: string): Promise<RequestModel> {
    let command: SaveRequestCommand = {requests: requests, collection: collection, request_name: requestName};
    return api.query(['save_request', command]);
  }

  copyToClipboard(value: string): Promise<null> {
    console.log('copy to clipboard');
    return api.query(['copy_to_clipboard', value]);
  }

  openFolderNative(path: string): Promise<null> {
    console.log('open folder native: ', path);
    return api.query(['open_folder_native', path]);
  }
}


export const backend = new Backend();


