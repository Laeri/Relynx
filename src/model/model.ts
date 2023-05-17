import { Workspace, Collection } from "../bindings";

export function newWorkspace(partial?: Partial<Workspace>): Workspace {
  if (!partial) {
    partial = {};
  }
  return { collections: [], ...partial };
}

export function newCollection(): Collection {
  return {
    name: "",
    path: "",
    description: "",
    import_warnings: [],
    current_env_name: ""
  };
}

