
import {create } from "zustand";
import {
    newRequestModel, newRequestResult,
    updatedRequestModel,
} from "../models/Request";
import Workspace = models.Workspace;
import {newWorkspace} from "../models/Workspace";
import Collection = models.Collection;
import RequestTree = models.RequestTree;
import RequestModel = models.RequestModel;
import Environment = models.Environment;
import {getUpdateRequestTreeWithRequest} from "../common/treeUtils";
import RequestResult = models.RequestResult;

interface RelynxState {

    currentRequest: RequestModel | undefined
    storeUpdateRequestAndTree: (requestModel: Partial<RequestModel>) => void

    currentEnvironment: Environment | undefined
    setCurrentEnvironment: (environment?: Environment) => void
    environments: Environment[]
    updateEnvironments: (environments: Environment[]) => void

    requestResult: RequestResult
    updateRequestResult: (requestResult: RequestResult) => void
    clearRequestResult: () => void

    // Workspace
    workspace: Workspace
    updateWorkspace: (partial: Partial<Workspace>) => void
    addCollection: (collection: Collection) => void
    removeCollection: (collection: Collection) => void

    currentCollection: Collection | undefined
    setCurrentCollection: (collection: Collection | undefined) => void

    setCurrentRequest: (requestModel?: RequestModel) => void
    setNewCurrentRequest: () => void

    // Tree
    requestTree: RequestTree
    updateRequestTree: (requestTree: RequestTree) => void,
}

//@TODO: Use immertype Callback = (state: State) => void;
//const setState = (fn: Callback) => set(produce(fn));

export const useRequestModelStore = create<RelynxState>((set) => {

    return {

        storeUpdateRequestAndTree: (partial: Partial<RequestModel>) => set((state: RelynxState) => {
            if (!state.currentRequest) {
                return {}
            }
            let newRequestModel = updatedRequestModel(state.currentRequest, partial)
            let newRequestTree = getUpdateRequestTreeWithRequest(state.requestTree, newRequestModel)

            return {
                ...state,
                currentRequest: newRequestModel,
                requestTree: newRequestTree
            }
        }),

        currentEnvironment: undefined,
        environments: [],

        updateEnvironments: (environments: Environment[]) => set((state: RelynxState) => {
            return {
                ...state,
                environments: environments
            }
        }),

        setCurrentEnvironment: (environment?: Environment) => set((state: RelynxState) => {
            return {
                ...state,
                currentEnvironment: environment
            }
        }),

        requestResult: new RequestResult(),

        updateRequestResult: (requestResult: RequestResult) => set((state: RelynxState) => {
            return {
                ...state,
                requestResult: requestResult
            }
        }),

        clearRequestResult: () => set((state: RelynxState) => ({
            ...state, requestResult: newRequestResult()
        })),

        setRequestModel: (newRequestModel: RequestModel) => set((state: RelynxState) => ({
            ...state,
            RequestModel: newRequestModel
        })),

        // Workspace
        workspace: newWorkspace(),

        updateWorkspace: (partial: Partial<Workspace>) => set((state: RelynxState) => {
            let newWorkspace = new Workspace({...partial});
            return {
                ...state,
                workspace: newWorkspace
            }
        }),

        addCollection: (collection: Collection) => set((state: RelynxState) => {
            let workspace = new Workspace(state.workspace)
            workspace.Collections.push(collection)
            return {
                ...state,
                workspace: workspace
            }
        }),

        removeCollection: (collection: Collection) => set((state: RelynxState) => {
            let workspace = new Workspace(state.workspace)
            workspace.Collections = workspace.Collections.filter((current: Collection) => current.Path !== collection.Path)
            workspace.Collections.push(collection)
            return {
                ...state,
                workspace: workspace
            }
        }),

        currentCollection: undefined,
        setCurrentCollection: (collection: Collection | undefined) => set((state: RelynxState) => {
            return {
                ...state,
                currentCollection: collection
            }
        }),

        currentRequest: newRequestModel(),
        setCurrentRequest: (requestModel?: RequestModel) => set((state: RelynxState) => {
            return {
                ...state,
                currentRequest: requestModel
            }

        }),
        setNewCurrentRequest: () => set((state: RelynxState) => {
            return {
                ...state,
                currentRequest: newRequestModel()
            }
        }),


        requestTree: new RequestTree(),

        updateRequestTree: (newTree: RequestTree) => set((state: RelynxState) => {
            return {
                ...state,
                requestTree: newTree
            }
        }),
    }
});

