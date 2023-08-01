import { Button } from "primereact/button";
import { Dialog } from "primereact/dialog";
import { OverlayPanel } from "primereact/overlaypanel";
import { useContext, useRef, useState } from "react";
import { useRequestModelStore } from "../../stores/requestStore";
import { catchError } from "../../common/errorhandling";
import { routes, ToastContext } from "../../App";
import { useNavigate } from "react-router";
import { Collection, Workspace } from "../../bindings";
import { backend } from "../../rpc";
import { EditCollectionNameModal } from "../../components/modals/EditCollectionNameModal";
import { create } from "react-modal-promise";
import { Tooltip } from "primereact/tooltip";

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
    }).catch(catchError);
  }


  const onCollectionEntryClicked = () => {
    // @ts-ignore
    op.current.hide();
    selectCollection(props.collection);
    navigate(routes.collection);
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
      // ignore
    }).catch(catchError);
  }

  // it can be null as the path_exists flag is kept from serializing to file, 
  // rspc marks it as optional in this case when creating the bindings
  // it has to be set explicitely to false, otherwise it is considered as true
  const canOpenCollection = (props.collection.path_exists ?? true);

  return (
    <>
      <div style={{ display: "flex", justifyContent: 'space-between', alignItems: 'center' }}>
        <Button disabled={!canOpenCollection} onClick={onCollectionEntryClicked} label={props.collection.name} className="p-button-text"
          style={{ 'display': 'block', marginTop: '5px', flexGrow: 1, textAlign: 'left' }} />
        {
          !canOpenCollection &&
          <>
            <Tooltip target={`.warn-collection`} />

            <i className="warn-collection pi p-error pi-exclamation-triangle"
              data-pr-tooltip={`The folder of this collection is missing. It might have either been renamed or removed.\nPrevious path: '${props.collection.path}'.\nRemove and readd the collection from the new location.`}
              data-pr-position="right"
              data-pr-at="right+5 top"
              data-pr-my="left center-2"
              style={{ fontSize: '1.2rem', marginRight: '30px', cursor: 'pointer' }}>
            </i>
          </>

        }
        <Button icon={'pi pi-ellipsis-h'} className={' p-button-text'} style={{ maxHeight: '10px' }}
          onClick={toggle} />

        <OverlayPanel ref={op}>
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>

            <Button style={{}} label="Rename" onClick={openRenameCollectionModal} outlined icon="pi pi-pencil" />
            <Button style={{ marginTop: '20px' }} label="Remove" onClick={openDialog} outlined icon="pi pi-trash" />
          </div>
        </OverlayPanel>
      </div>

      <Dialog header="Remove Collection" visible={dialogVisible} style={{ padding: 0 }} footer={
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
