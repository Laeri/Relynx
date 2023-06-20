import { Button } from "primereact/button";
import { Dialog } from "primereact/dialog";
import { OverlayPanel } from "primereact/overlaypanel";
import { useContext, useRef, useState } from "react";
import { useRequestModelStore } from "../stores/requestStore";
import { catchError } from "../common/errorhandling";
import { ToastContext } from "../App";
import { useNavigate } from "react-router";
import { Collection, Workspace } from "../bindings";
import { backend } from "../rpc";
import { EditCollectionNameModal } from "../components/modals/EditCollectionNameModal";
import { create } from "react-modal-promise";

interface ComponentProps {
  collection: Collection
}

export function CollectionEntry(props: ComponentProps) {

  const setCurrentCollection = useRequestModelStore((state) => state.setCurrentCollection);
  // @TODO: const setNewCurrentRequest = useRequestModelStore((state) => state.setNewCurrentRequest)
  const updateWorkspace = useRequestModelStore((state) => state.updateWorkspace);
  const toast = useContext(ToastContext);

  const navigate = useNavigate();

  const selectCollection = (collection: Collection) => {
    setCurrentCollection(collection);
    // @TODO: setNewCurrentRequest();
  }

  const op = useRef(null);
  const toggle = (e: any) => { // @ts-ignore
    op.current.toggle(e)
  }

  const [dialogVisible, setDialogVisible] = useState<boolean>(false);

  const openDialog = () => {
    // @ts-ignore
    op?.current?.hide();
    setDialogVisible(true)
  }

  const onHide = () => {
    setDialogVisible(false)
  }

  const doRemove = () => {
    backend.removeCollection(props.collection).then((workspace: Workspace) => {
      updateWorkspace(workspace);
      onHide();
    }).catch(catchError(toast));
  }


  const onCollectionEntryClicked = () => {
    // @ts-ignore
    op.current.hide();
    selectCollection(props.collection);
    navigate('/collection');
  }

  const openRenameCollectionModal = () => {
    // @ts-ignore
    op.current.hide();
    const modalPromise = create(({ onResolve, onReject, isOpen }) => {
      return <EditCollectionNameModal collection={props.collection} isOpen={isOpen} onResolve={onResolve} onReject={() => onReject()} />
    });

    modalPromise().then((newName?: string) => {
      if (newName) {
        toast.showSuccess(`Renamed collection to: '${newName}`, "");
      }
      console.log('modal promisiert');
      // ignore
    }).catch(catchError);
  }

  return (
    <>
      <div style={{ display: "flex", justifyContent: 'space-between', alignItems: 'center' }}>
        <Button onClick={onCollectionEntryClicked} label={props.collection.name} className="p-button-text"
          style={{ 'display': 'block', marginTop: '5px', flexGrow: 1, textAlign: 'left' }} />
        <Button icon={'pi pi-ellipsis-h'} className={' p-button-text'} style={{ maxHeight: '10px' }}
          onClick={toggle} />

        <OverlayPanel ref={op}>
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>

            <Button label="Remove" onClick={openDialog} outlined icon="pi pi-trash" />
            <Button style={{ marginTop: '20px' }} label="Rename" onClick={openRenameCollectionModal} outlined icon="pi pi-pencil" />
          </div>
        </OverlayPanel>
      </div>

      <Dialog header="Remove Collection" visible={dialogVisible} style={{ width: '50vw', padding: 0 }} footer={
        <div>
          <Button label="No" icon="pi pi-times" onClick={() => onHide()} className="p-button-text" />
          <Button label="Remove" icon="pi pi-check" onClick={() => doRemove()} autoFocus />
        </div>
      } onHide={() => onHide()}>
        <div>
          <p>
            Are you sure you want to remove the collection <b>"{props.collection.name}"</b>?
          </p>
          <br />
          {/*TODO: option to remove collection also from file system?*/}
          <p>
            The collection is only removed from this program. The collection folder with all its files will
            remain on your system. So you might want to delete it manually.
          </p>
        </div>

      </Dialog>
    </>
  )
}
