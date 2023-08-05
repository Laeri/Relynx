import { create } from "react-modal-promise";
import { AddExistingCollectionsModal } from "../components/modals/AddExistingCollectionsModal";
import { useRequestModelStore } from "../stores/requestStore";
import { Workspace, Collection, AddCollectionsResult, ImportCollectionResult, RequestModel } from "../bindings";
import { newWorkspace, newCollection } from "../model/model";
import { backend } from "../rpc";
import { catchError } from "./errorhandling";
import { ExternalToast, ToastContext } from "../App";
import { ImportCollectionModal, ImportType } from "../components/modals/ImportCollectionModal";
import { CreateCollectionModal } from "../components/modals/CreateCollectionModal";
import { ImportResultModal } from "../components/modals/ImportResultModal";
import { ImportPostmanModal } from "../components/modals/ImportPostmanModal";
import { ImportJetbrainsHttpFolder } from "../components/modals/ImportJetbrainsHttpFolder";
import { ErrorReportingModal } from "../components/modals/ErrorReportingModal";
import { EditRequestNameModal } from "../components/modals/EditRequestNameModal";
import { AddCookieHeaderModal } from "../components/modals/AddCookieHeaderModal";


export const addCollectionToWorkspace = (newCollection: Collection) => {
  let updatedWorkspace = newWorkspace();
  const workspace = useRequestModelStore.getState().workspace
  const updateWorkspaceInStore = useRequestModelStore.getState().updateWorkspace;
  updatedWorkspace.collections = [...workspace.collections, newCollection];
  backend.updateWorkspace(updatedWorkspace).then(() => {
    updateWorkspaceInStore(updatedWorkspace);
  }).catch(catchError);
}

export const openCreateCollectionModal = (): Promise<Collection | undefined> => {
  const createCollectionModal = create(({ isOpen, onResolve, onReject }) => {
    return <CreateCollectionModal isOpen={isOpen} onResolve={onResolve} onReject={onReject} />
  });

  return createCollectionModal().then((result?: { collectionName: string, collectionPath: string }) => {
    if (!result) {
      return
    }
    let collection: Collection = newCollection();
    collection.name = result.collectionName;
    collection.path = result.collectionPath;

    addCollectionToWorkspace(collection);
    return collection
  });
}

export const openAddExistingCollectionsModal = (workspace: Workspace, toast: ToastContext): Promise<void | AddCollectionsResult> => {
  const addCollectionModal = create(({ isOpen, onResolve, onReject }) => {
    return <AddExistingCollectionsModal isOpen={isOpen} onResolve={onResolve} onReject={onReject} />
  })

  return addCollectionModal().then((result?: { collectionPath: string }): Promise<void | AddCollectionsResult> => {
    if (!result) {
      return Promise.resolve();
    }

    return backend.addExistingCollections(result.collectionPath, workspace).then((result: AddCollectionsResult) => {
      if (result.num_imported == 0) {
        toast.showWarn("No results found", "No collections found that can be imported. Try another location.");
        return { collection_names: [], workspace: result.workspace, any_collections_found: false, num_imported: 0, errored_collections: result.errored_collections } as AddCollectionsResult
      }
      if (!result.workspace) {
        toast.showError("", "Could not add collections to workspace.");
        return { collection_names: [], workspace: result.workspace, num_imported: 0, errored_collections: result.errored_collections } as AddCollectionsResult
      }
      useRequestModelStore.getState().updateWorkspace(result.workspace);
      if (result.errored_collections && result.errored_collections.length > 0) {
        let paths = result.errored_collections;
        toast.showError("Collections", `${paths.join(',')} could not be added successfully`);
        toast.showInfo(`${result.num_imported} collections have been added`, "");
      } else {
        if (result.num_imported == 1) {
          toast.showSuccess(`The collection '${result.collection_names[0]}' has been added successfully`, "");
        } else {
          toast.showSuccess(`${result.num_imported} collections have been added`, "");
        }
      }
      return Promise.resolve(result)
    }).catch(catchError);
  });
}

const doPostmanImport = (toast: ToastContext, workspace: Workspace, collectionName: string) => {
  const postmanCollectionModal = create(({ isOpen, onResolve, onReject }) => {
    return <ImportPostmanModal isOpen={isOpen} onResolve={onResolve} onReject={onReject} />
  });

  postmanCollectionModal().then((result?: { collectionPath: string, importCollectionFilepath: string }) => {
    if (!result) {
      return
    }
    backend.importPostmanCollection(workspace, result.importCollectionFilepath, result.collectionPath).then((importResult: ImportCollectionResult) => {
      addCollectionToWorkspace(importResult.collection as Collection);
      if (importResult.collection.import_warnings.length > 0) {
        const importResultModal = create(({ isOpen, onResolve, onReject }) => {
          return <ImportResultModal isOpen={isOpen} onResolve={onResolve} onReject={onReject}
            importCollectionResult={importResult} />
        });
        importResultModal().then(() => {
          // ignore
        }).catch((_ignored: any) => {
        });
      } else {
        toast.showSuccess(`Collection: \"${importResult.collection?.name}\" has been imported`, "")
      }
    }).catch(catchError);
  })
}

const doJetbrainsHttpImport = (toast: ToastContext, workspace: Workspace, collectionName: string) => {
  const jetbrainsImportModal = create(({ isOpen, onResolve, onReject }) => {
    return <ImportJetbrainsHttpFolder isOpen={isOpen} onResolve={onResolve} onReject={onReject} />
  });

  jetbrainsImportModal().then((result?: { collectionPath: string }) => {
    if (!result) {
      return
    }
    // TODO: what about the name?
    backend.importJetbrainsFolder(workspace, result.collectionPath, collectionName).then((newWorkspace: Workspace) => {
      const updateWorkspaceInStore = useRequestModelStore.getState().updateWorkspace;
      updateWorkspaceInStore(newWorkspace);
      toast.showSuccess(`Collection: '${collectionName}' has been imported`, "")
    }).catch(catchError);
  }).catch(catchError);
}




export const openImportCollectionModal = (workspace: Workspace) => {

  const toast = ExternalToast;

  const importCollectionModal = create(({ isOpen, onResolve, onReject }) => {
    return <ImportCollectionModal isOpen={isOpen} onResolve={onResolve} onReject={onReject} />
  });

  importCollectionModal().then((result?: { importType: ImportType, collectionName: string }) => {
    if (!result) {
      return
    }

    if (result.importType === ImportType.Postman) {
      doPostmanImport(toast as ToastContext, workspace, result.collectionName);
    } else if (result.importType === ImportType.JetbrainsHttpRest) {
      doJetbrainsHttpImport(toast as ToastContext, workspace, result.collectionName);
    }
  }).catch(catchError)
}

export const openErrorReportingModal = (title: string, detail: string) => {
  const modalPromise = create(({ onResolve, onReject, isOpen }) => {
    return <ErrorReportingModal title={title} detail={detail} isOpen={isOpen} onResolve={onResolve} onReject={() => onReject()} />
  });
  modalPromise().then(() => {
  })

}

export const openEditRequestNameModal = (request: RequestModel, collection: Collection, updateRequest: (newName: string) => Promise<void>) => {
  const modalPromise = create(({ onResolve, onReject, isOpen }) => {
    return <EditRequestNameModal collection={collection} request={request} updateRequest={updateRequest} isOpen={isOpen} onResolve={onResolve} onReject={() => onReject()} />
  });
  modalPromise().then(() => {
  })
}

export const openAddCookieModal = (): Promise<[key: string, value: string] | undefined> => {
  const modalPromise = create(({ onResolve, onReject, isOpen }) => {
    return <AddCookieHeaderModal isOpen={isOpen} onResolve={onResolve} onReject={() => onReject()} />
  });
  return modalPromise()
}

