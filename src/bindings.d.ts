// This file was generated by [rspc](https://github.com/oscartbeaumont/rspc). Do not edit this file manually.

export type Procedures = {
    queries: 
        { key: "load_workspace", input: never, result: Workspace },
    mutations: never,
    subscriptions: never
};

export type Workspace = { collections: Collection[] }

export type Collection = { name: string; path: string; current_env_name: string; description: string; import_warnings: ImportWarning[] }

export type ImportWarning<> = null
