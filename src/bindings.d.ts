// This file was generated by [rspc](https://github.com/oscartbeaumont/rspc). Do not edit this file manually.

export type Procedures = {
    queries: 
        { key: "add_existing_collections", input: AddExistingCollectionsParams, result: AddCollectionsResult } | 
        { key: "add_group_node", input: AddGroupNodeParams, result: RequestTreeNode } | 
        { key: "add_request_node", input: AddRequestNodeParams, result: RequestTreeNode } | 
        { key: "copy_to_clipboard", input: string, result: null } | 
        { key: "delete_node", input: DeleteNodeParams, result: null } | 
        { key: "drag_and_drop", input: DragAndDropParams, result: DragAndDropResult } | 
        { key: "import_postman_collection", input: ImportPostmanCommandParams, result: ImportCollectionResult } | 
        { key: "is_directory_empty", input: string, result: boolean } | 
        { key: "load_requests_for_collection", input: Collection, result: LoadRequestsResult } | 
        { key: "load_workspace", input: never, result: Workspace } | 
        { key: "open_folder_native", input: string, result: null } | 
        { key: "remove_collection", input: Collection, result: Workspace } | 
        { key: "reorder_nodes_within_parent", input: ReorderNodesParams, result: RequestTreeNode } | 
        { key: "run_request", input: RunRequestCommand, result: RequestResult } | 
        { key: "save_request", input: SaveRequestCommand, result: RequestModel } | 
        { key: "select_directory", input: never, result: string } | 
        { key: "select_file", input: never, result: string } | 
        { key: "update_workspace", input: Workspace, result: null },
    mutations: never,
    subscriptions: never
};

export type Replaced<T> = { value: T; is_replaced: boolean }

export type RequestResult = { result: string; status_code: string; total_time: number; total_result_size: number; content_type: string }

export type ImportPostmanCommandParams = { workspace: Workspace; import_postman_path: string; import_result_path: string }

export type SaveRequestCommand = { requests: RequestModel[]; collection: Collection; request_name: string }

export type LoadRequestsResult = { request_tree: RequestTree; errs: FrontendError[] }

export type QueryParam = { key: string; value: string; active: boolean }

export type RequestModel = { id: string; name: string; description: string; method: HttpMethod; url: string; query_params: QueryParam[]; headers: Header[]; body: RequestBody; rest_file_path: string; http_version: Replaced<HttpVersion>; settings: RequestSettings }

export type RequestSettings = { no_redirect: boolean | null; no_log: boolean | null; no_cookie_jar: boolean | null; use_os_credentials: boolean | null }

export type RunRequestCommand = { request: RequestModel; environment: Environment | null }

export type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "TRACE" | "OPTIONS" | "CONNECT" | { CUSTOM: string }

export type DeleteNodeParams = { collection: Collection; node: RequestTreeNode }

export type ImportCollectionResult = { collection: Collection; import_warnings: ImportWarning[] }

export type AddExistingCollectionsParams = { path: string; workspace: Workspace }

export type Collection = { name: string; path: string; current_env_name: string; description: string; import_warnings: ImportWarning[] }

export type Multipart = { name: string; data: DataSource<string>; fields: DispositionField[]; headers: Header[] }

export type RequestTreeNode = { id: string; name: string; request: RequestModel | null; children: RequestTreeNode[]; filepath: string; is_file_group: boolean }

export type AddGroupNodeParams = { collection: Collection; parent: RequestTreeNode; group_name: string }

export type DisplayErrorKind = "Generic" | "LoadWorkspaceError" | "ReadWorkspaceFileError" | "DeserializeWorkspaceError" | "SerializeWorkspaceError" | "SaveWorkspaceError" | "NoPathChosen" | "ImportPostmanError" | "ParseError" | "InvalidOpenPath" | "CopyToClipboardError" | "RequestFileAlreadyExists"

export type EnvVarDescription = { env_var_name: string; description: string; is_secret: boolean }

export type ImportWarning = { rest_file_path: string; request_name: string }

export type Header = { key: string; value: string; active: boolean }

export type ReorderNodesParams = { collection: Collection; drag_node: RequestTreeNode; drop_node: RequestTreeNode; drop_index: number }

export type Workspace = { collections: Collection[] }

export type DragAndDropParams = { collection: Collection; drag_node_parent: RequestTreeNode; drag_node: RequestTreeNode; drop_node: RequestTreeNode; drop_index: number }

export type DragAndDropResult = { new_drop_node: RequestTreeNode; remove_drag_node_parent: boolean }

export type EnvironmentSecret = { name: string; initial_value: string; current_value: string; description: string; persist_to_file: boolean }

export type HttpVersion = { major: number; minor: number }

export type DataSource<T> = { Raw: T } | { FromFilepath: T }

export type RequestBody = "None" | { Multipart: { boundary: string; parts: Multipart[] } } | { Text: { data: DataSource<string> } }

export type AddRequestNodeParams = { collection: Collection; parent: RequestTreeNode; new_request: RequestModel; requests_in_same_file: RequestModel[] }

export type EnvironmentVariable = { name: string; initial_value: string; current_value: string; description: string }

export type AddCollectionsResult = { workspace: Workspace; any_collections_found: boolean; num_imported: number; errored_collections: string[] }

export type FrontendError = { kind: DisplayErrorKind; message: string | null }

export type DispositionField = { key: string; value: string }

export type Environment = { name: string; variables: EnvironmentVariable[]; secrets: EnvironmentSecret[]; env_var_descriptions: EnvVarDescription[] }

export type RequestTree = { root: RequestTreeNode }
