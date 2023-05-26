import { create } from "zustand";
import { Workspace, Collection, Environment, RequestModel, RequestResult } from "../bindings.d";
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
  currentEnvironment?: Environment,
  environments: Environment[],


  setCurrentEnvironment: (environment?: Environment) => void,

  requestResult?: RequestResult,

  clearRequestResult: () => void,

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
  }
});

