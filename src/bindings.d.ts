// This file was generated by [rspc](https://github.com/oscartbeaumont/rspc). Do not edit this file manually.

export type Procedures = {
    queries: 
        { key: "add_existing_collections", input: AddExistingCollectionsParams, result: AddCollectionsResult } | 
        { key: "add_group_node", input: AddGroupNodeParams, result: RequestTreeNode } | 
        { key: "add_request_node", input: AddRequestNodeParams, result: RequestTreeNode } | 
        { key: "choose_file_relative_to", input: ChooseFileRelativeToParams, result: string } | 
        { key: "copy_to_clipboard", input: string, result: null } | 
        { key: "delete_node", input: DeleteNodeParams, result: null } | 
        { key: "drag_and_drop", input: DragAndDropParams, result: DragAndDropResult } | 
        { key: "get_app_environment", input: never, result: AppEnvironment } | 
        { key: "get_response_filepath", input: string, result: string } | 
        { key: "hide_group", input: string, result: null } | 
        { key: "import_jetbrains_folder", input: ImportJetbrainsHttpFolderParams, result: Workspace } | 
        { key: "import_postman_collection", input: ImportPostmanCommandParams, result: ImportCollectionResult } | 
        { key: "is_directory_empty", input: string, result: boolean } | 
        { key: "is_signature_valid", input: LicenseData, result: boolean } | 
        { key: "load_environments", input: string, result: Environment[] } | 
        { key: "load_license_data", input: never, result: LicenseData } | 
        { key: "load_requests_for_collection", input: Collection, result: LoadRequestsResult } | 
        { key: "load_workspace", input: never, result: Workspace } | 
        { key: "open_folder_native", input: string, result: null } | 
        { key: "remove_collection", input: Collection, result: Workspace } | 
        { key: "rename_group", input: RenameGroupParams, result: string } | 
        { key: "reorder_nodes_within_parent", input: ReorderNodesParams, result: RequestTreeNode } | 
        { key: "run_request", input: RunRequestCommand, result: RequestResult } | 
        { key: "save_environments", input: SaveEnvironmentsParams, result: null } | 
        { key: "save_request", input: SaveRequestCommand, result: string } | 
        { key: "select_directory", input: never, result: string } | 
        { key: "select_file", input: never, result: string } | 
        { key: "update_workspace", input: Workspace, result: null } | 
        { key: "validate_group_name", input: ValidateGroupNameParams, result: ValidateGroupNameResult } | 
        { key: "validate_response_filepath", input: string, result: boolean },
    mutations: 
        { key: "save_license_data", input: LicenseData, result: null },
    subscriptions: never
};

export type RequestModel = { id: string; name: string; description: string; method: HttpMethod; url: string; query_params: QueryParam[]; headers: Header[]; body: RequestBody; rest_file_path: string; http_version: Replaced<HttpVersion>; settings: RequestSettings; redirect_response: RedirectResponse }

export type ValidateGroupNameParams = { old_path: string; new_name: string }

export type ReorderNodesParams = { collection: Collection; drag_node: RequestTreeNode; drop_node: RequestTreeNode; drop_index: number }

export type LicenseData = { license_key?: string | null; license_signature?: string | null; license_start?: string | null }

export type SaveEnvironmentsParams = { collection_path: string; environments: Environment[] }

export type EnvironmentSecret = { name: string; initial_value: string; current_value: string | null; description: string | null; persist_to_file: boolean }

export type ValidateGroupNameResult = { sanitized_name: string; new_path: string; path_exists_already: boolean }

export type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "TRACE" | "OPTIONS" | "CONNECT" | { CUSTOM: string }

export type SaveRequestCommand = { requests: RequestModel[]; collection: Collection; request_name: string }

export type Workspace = { collections: Collection[] }

export type RequestSettings = { no_redirect: boolean | null; no_log: boolean | null; no_cookie_jar: boolean | null }

export type RenameGroupParams = { collection_path: string; old_path: string; new_name: string }

export type FrontendError = { kind: DisplayErrorKind; message: string | null }

export type ImportWarning = { rest_file_path: string; is_group: boolean; message: string | null; severity: MessageSeverity | null }

export type DataSource<T> = { Raw: T } | { FromFilepath: string }

export type RunRequestCommand = { request: RequestModel; environment: Environment | null }

export type DeleteNodeParams = { collection: Collection; node: RequestTreeNode; file_node: RequestTreeNode | null }

export type AddCollectionsResult = { workspace: Workspace; num_imported: number; errored_collections: string[] }

export type RedirectResponse = { save_response: boolean; save_path: string | null; overwrite: boolean }

export type RequestTree = { root: RequestTreeNode }

export type ImportJetbrainsHttpFolderParams = { workspace: Workspace; import_jetbrains_folder: string; collection_name: string }

export type RequestBody = "None" | { Multipart: { boundary: string; parts: Multipart[] } } | { UrlEncoded: { url_encoded_params: UrlEncodedParam[] } } | { Raw: { data: DataSource<string> } }

export type Environment = { name: string; variables: EnvironmentVariable[]; secrets: EnvironmentSecret[] }

export type AddExistingCollectionsParams = { path: string; workspace: Workspace }

export type UrlEncodedParam = { key: string; value: string }

export type Collection = { name: string; path: string; current_env_name: string; description: string; import_warnings: ImportWarning[]; path_exists?: boolean }

export type AddGroupNodeParams = { collection: Collection; parent: RequestTreeNode; group_name: string }

export type Replaced<T> = { value: T; is_replaced: boolean }

export type AddRequestNodeParams = { collection: Collection; parent: RequestTreeNode; request_name: string; requests_in_same_file: RequestModel[] }

export type HttpVersion = { major: number; minor: number }

export type DragAndDropResult = { new_drop_node: RequestTreeNode; remove_drag_node_parent: boolean }

export type QueryParam = { key: string; value: string; active: boolean }

export type Multipart = { data: DataSource<string>; disposition: DispositionField; headers: Header[] }

export type AppEnvironment = "DEVELOPMENT" | "PRODUCTION"

export type DispositionField = { name: string; filename: string | null; filename_star: string | null }

export type ImportCollectionResult = { collection: Collection }

export type DisplayErrorKind = "Generic" | "LoadWorkspaceError" | "ReadWorkspaceFileError" | "DeserializeWorkspaceError" | "SerializeWorkspaceError" | "SaveWorkspaceError" | "NoPathChosen" | "ImportPostmanError" | "ParseError" | "InvalidOpenPath" | "CopyToClipboardError" | "RequestFileAlreadyExists" | "NodeDeleteError" | "SaveRequestError" | "RemoveOldRequestFile" | "AddGroupNodeError" | "DragAndDropError" | "InvalidCollectionConfig" | "ReorderError" | "UnsupportedImportFormat" | "ImportSerializeError" | "LoadEnvironmentsError" | "SaveEnvironmentsError" | "RequestFileMissing" | "CurlError" | "RequestSendError"

export type RequestResult = { id: string; result: string; status_code: string; total_time: number; total_result_size: number; content_type: string | null; warnings: string[]; result_file: string | null; result_file_folder: string | null }

export type DragAndDropParams = { collection: Collection; drag_node_parent: RequestTreeNode; drag_node: RequestTreeNode; drop_node: RequestTreeNode; drop_index: number }

export type Header = { key: string; value: string; active: boolean }

export type LoadRequestsResult = { request_tree: RequestTree; errs: FrontendError[] }

export type RequestTreeNode = { id: string; name: string; request: RequestModel | null; children: RequestTreeNode[]; filepath: string; is_file_group: boolean }

export type EnvironmentVariable = { name: string; initial_value: string; current_value: string | null; description: string | null }

export type MessageSeverity = "warn" | "warn" | "success" | "error"

export type ImportPostmanCommandParams = { workspace: Workspace; import_postman_path: string; import_result_path: string }

export type ChooseFileRelativeToParams = { base_path: string }
