import { create } from "zustand";
import { Workspace, Collection, Environment, RequestModel, RequestResult, RequestTree } from "../bindings.d";
import { newWorkspace } from "../model/model";

interface RelynxState {

  // Workspace
  workspace: Workspace
  updateWorkspace: (partial: Partial<Workspace>) => void
  addCollection: (collection: Collection) => void
  removeCollection: (collection: Collection) => void,

  currentCollection?: Collection,

  setCurrentCollection: (collection?: Collection) => void,
  currentRequest?: RequestModel,
  setCurrentRequest: (request?: RequestModel) => void,
  currentEnvironment?: Environment,
  environments: Environment[],


  setCurrentEnvironment: (environment?: Environment) => void,

  requestResult?: RequestResult,

  clearRequestResult: () => void,

  requestTree?: RequestTree,

  updateRequestTree: (requestTree: RequestTree) => void

}

//@TODO: Use immertype Callback = (state: State) => void;
//const setState = (fn: Callback) => set(produce(fn));

export const useRequestModelStore = create<RelynxState>((set) => {

  return {

    // Workspace
    workspace: newWorkspace(),
    currentCollection: undefined,

    updateWorkspace: (partial: Partial<Workspace>) => set((state: RelynxState) => {
      let workspace = newWorkspace({ ...partial });
      return {
        ...state,
        workspace: workspace
      }
    }),

    addCollection: (collection: Collection) => set((state: RelynxState) => {
      // @TODO
      let workspace = newWorkspace(state.workspace)
      workspace.collections.push(collection)
      return {
        ...state,
        workspace: workspace
      }
      return state
    }),

    removeCollection: (collection: Collection) => set((state: RelynxState) => {
      let workspace = newWorkspace(state.workspace)
      // @TODO: use id on collection?
      workspace.collections = workspace.collections.filter((current: Collection) => current.path !== collection.path)
      workspace.collections.push(collection)
      return {
        ...state,
        workspace: workspace
      }
      return state
    }),
    setCurrentCollection: (collection?: Collection) => set((state: RelynxState) => {
      return {
        ...state,
        currentCollection: collection
      }
    }),

    currentRequest: undefined,
    setCurrentRequest: (request?: RequestModel) => set((state: RelynxState) => {
      return {
        ...state,
        currentRequest: request
      };
    }),

    currentEnvironment: undefined,
    environments: [],
    setCurrentEnvironment: (environment?: Environment) => set((state: RelynxState) => {
      return {
        ...state,
        currentEnvironment: environment
      }
    }),

    requestResult: undefined,

    clearRequestResult: () => set((state: RelynxState) => ({
      ...state, requestResult: undefined
    })),

    requestTree: undefined,
    updateRequestTree: (requestTree: RequestTree) => set((state: RelynxState) => {
      return {
        ...state, requestTree: requestTree
      };

    })
  }
});

