import { create } from "react-modal-promise";
import { AddExistingCollectionsModal } from "../components/modals/AddExistingCollectionsModal";
import { useRequestModelStore } from "../stores/requestStore";
import { Workspace, Collection, AddCollectionsResult, ImportCollectionResult } from "../bindings";
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


export const addCollectionToWorkspace = (newCollection: Collection) => {
  const toast = ExternalToast;
  let updatedWorkspace = newWorkspace();
  const workspace = useRequestModelStore.getState().workspace
  const updateWorkspaceInStore = useRequestModelStore.getState().updateWorkspace;
  updatedWorkspace.collections = [...workspace.collections, newCollection];
  backend.updateWorkspace(updatedWorkspace).then(() => {
    updateWorkspaceInStore(updatedWorkspace);
  }).catch(catchError(toast));
}

export const openCreateCollectionModal = (workspace: Workspace): Promise<Collection | undefined> => {
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
        return { workspace: result.workspace, any_collections_found: false, num_imported: 0, errored_collections: result.errored_collections } as AddCollectionsResult
      }
      if (!result.workspace) {
        toast.showError("", "Could not add collections to workspace.");
        return { workspace: result.workspace, num_imported: 0, errored_collections: result.errored_collections } as AddCollectionsResult
      }
      useRequestModelStore.getState().updateWorkspace(result.workspace);
      if (result.errored_collections && result.errored_collections.length > 0) {
        let paths = result.errored_collections;
        toast.showError("Collections", `${paths.join(',')} could not be added successfully`);
        toast.showInfo(`${result.num_imported} collections have been added`, "");
      } else {
        if (result.num_imported == 1) {
          toast.showSuccess(`${result.num_imported} collection has been added`, "");
        } else {
          toast.showSuccess(`${result.num_imported} collections have been added`, "");
        }
      }
      return Promise.resolve(result)
    }).catch((err: any) => {
      catchError(ExternalToast)(err);
    });
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
    })
  }).catch(catchError);
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
      doPostmanImport(toast, workspace, result.collectionName);
    } else if (result.importType === ImportType.JetbrainsHttpRest) {
      doJetbrainsHttpImport(toast, workspace, result.collectionName);
    }
  }).catch(catchError(toast))
}

export const openErrorReportingModal = (errorMsg: string) => {
  const modalPromise = create(({ onResolve, onReject, isOpen }) => {
    return <ErrorReportingModal errorMsg={errorMsg} isOpen={isOpen} onResolve={onResolve} onReject={() => onReject()} />
  });
  modalPromise().then(() => {
  })

}
