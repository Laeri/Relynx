import { create } from "zustand";
import { Workspace, Collection } from "../bindings.d";
import { newWorkspace } from "../model/model";

interface RelynxState {

  // Workspace
  workspace: Workspace
    updateWorkspace: (partial: Partial<Workspace>) => void
    addCollection: (collection: Collection) => void
    removeCollection: (collection: Collection) => void
}

//@TODO: Use immertype Callback = (state: State) => void;
//const setState = (fn: Callback) => set(produce(fn));

export const useRequestModelStore = create<RelynxState>((set) => {

  return {

    // Workspace
    workspace: newWorkspace(),

    updateWorkspace: (partial: Partial<Workspace>) => set((state: RelynxState) => {
      let workspace = newWorkspace({ ...partial });
      return {
        ...state,
        workspace: workspace
      }
    }),

    addCollection: (collection: Collection) => set((state: RelynxState) => {
      // @TODO
      /* let workspace = new Workspace(state.workspace)
      workspace.Collections.push(collection)
      return {
        ...state,
        workspace: workspace
      } */
      return state
    }),

    // @TODO
    removeCollection: (collection: Collection) => set((state: RelynxState) => {
      /*       let workspace = new Workspace(state.workspace)
            workspace.Collections = workspace.Collections.filter((current: Collection) => current.Path !== collection.Path)
            workspace.Collections.push(collection)
            return {
              ...state,
              workspace: workspace
            } */
      return state
    }),

  }
});

