import { create } from "react-modal-promise";
import { AddCollectionModal } from "../components/modals/AddCollectionModal";
import { useRequestModelStore } from "../stores/requestStore";
import { Workspace, Collection, AddCollectionsResult, ImportCollectionResult } from "../bindings";
import { newWorkspace, newCollection as newCollectionFunc } from "../model/model";
import { backend } from "../rpc";
import { catchError } from "./errorhandling";
import { ExternalToast, ToastContext } from "../App";
import { ImportCollectionModal } from "../components/modals/ImportCollectionModal";
import { CreateCollectionModal } from "../components/modals/CreateCollectionModal";
// @TODO import ImportCollectionResult = collectionImport.ImportCollectionResult;
import { ImportResultModal } from "../components/modals/ImportResultModal";


export const addCollectionToWorkspace = (newCollection: Collection) => {
  const toast = ExternalToast;
  let updatedWorkspace = newWorkspace();
  const workspace = useRequestModelStore.getState().workspace
  updatedWorkspace.collections = [...workspace.collections, newCollection];
  backend.updateWorkspace(updatedWorkspace).catch(catchError(toast));
}

export const openCreateCollectionModal = (): Promise<Collection | undefined> => {
  const createCollectionModal = create(({ isOpen, onResolve, onReject }) => {
    return <CreateCollectionModal isOpen={isOpen} onResolve={onResolve} onReject={onReject} />
  })

  return createCollectionModal().then((result?: { collectionName: string, collectionPath: string }) => {
    if (!result) {
      return
    }
    let newCollection: Collection = newCollectionFunc();
    newCollection.name = result.collectionName;
    newCollection.path = result.collectionPath;

    const addCollectionToStore = useRequestModelStore.getState().addCollection

    addCollectionToStore(newCollection);
    addCollectionToWorkspace(newCollection);
    return newCollection
  });
}

export const openAddExistingCollectionModal = (toast: ToastContext): Promise<{ workspace: Workspace | undefined, any_collections_found: boolean, cancelled: boolean }> => {
  const addCollectionModal = create(({ isOpen, onResolve, onReject }) => {
    return <AddCollectionModal isOpen={isOpen} onResolve={onResolve} onReject={onReject} />
  })

  return addCollectionModal().then((result?: { collectionPath: string }): AddCollectionsResult => {
    return backend.addExistingCollections(result.collectionPath).then((result: AddCollectionsResult) => {
      if (!result.any_collections_found) {
        toast.showWarn("No results found", "No collections found that can be imported. Try another location.")
        return { workspace: result.workspace, any_collections_found: false, cancelled: false }
      }
      if (!result.workspace) {
        toast.showError("", "Could not add collections to workspace.")
        return { workspace: result.workspace, any_collections_found: result.any_collections_found, cancelled: false }
      }
      useRequestModelStore.getState().updateWorkspace(result.workspace)
      if (result.errored_collections && result.errored_collections.length > 0) {
        let paths = result.errored_collections // @TODO import .map((erroredCollection: ErroredCollectionResult) => erroredCollection.Path)
        toast.showError("Error importing collections", `${paths.join(',')} could not be added successfully`)
        toast.showInfo("Collections Imported", `${result.num_imported} collections have been added`)
      } else {
        toast.showSuccess("Collections Imported", `${result.num_imported} collections have been added`)
      }
      return { workspace: result.workspace, any_collections_found: result.any_collections_found, cancelled: false }
    }).catch((err: any) => {
      catchError(ExternalToast)(err);
    });
  });
}

export const openImportCollectionModal = (workspace: Workspace) => {

  const toast = ExternalToast;


  const importCollectionModal = create(({ isOpen, onResolve, onReject }) => {
    return <ImportCollectionModal isOpen={isOpen} onResolve={onResolve} onReject={onReject} />
  });

  importCollectionModal().then((result?: { collectionPath: string, collectionName: string, importCollectionFilepath: string }) => {
    if (!result) {
      return
    }

    const addCollectionToStore = useRequestModelStore.getState().addCollection

    // TODO: what about the name?
    backend.importPostmanCollection(workspace, result.importCollectionFilepath, result.collectionPath).then((importResult: ImportCollectionResult) => {
      addCollectionToStore(importResult.collection as Collection);
      addCollectionToWorkspace(importResult.collection as Collection);

      if (importResult.import_warnings.length > 0) {
        const importResultModal = create(({ isOpen, onResolve, onReject }) => {
          return <ImportResultModal isOpen={isOpen} onResolve={onResolve} onReject={onReject}
            importCollectionResult={importResult} />
        });
        importResultModal().then(() => {
          // ignore
          // @TODO: open collection after import
        }).catch((_ignored: any) => {
          // ignore
        });

      } else {
        toast.showSuccess(`Collection: \"${importResult.collection?.name}\" has been imported`, "")
      }

    }).catch(catchError(toast))
  })
}