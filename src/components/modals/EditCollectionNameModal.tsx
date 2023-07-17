import { Dialog } from "primereact/dialog";
import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";
import { useContext, useState } from "react";
import { onCtrlEnter } from "../../common/eventhandling";
import { backend } from "../../rpc";
import { Collection } from "../../bindings";
import { catchError } from "../../common/errorhandling";
import { RelynxState, useRequestModelStore } from "../../stores/requestStore";


interface ComponentProps {
  isOpen: boolean,
  onResolve: (newName?: string) => void,
  onReject: () => void,
  collection: Collection
}

export function EditCollectionNameModal(props: ComponentProps) {

  const [collectionName, setCollectionName] = useState<string>(props.collection.name);
  const [nameExists, setNameExists] = useState<boolean>(false);

  const workspace = useRequestModelStore((state: RelynxState) => state.workspace);
  const updateWorkspaceStore = useRequestModelStore((state: RelynxState) => state.updateWorkspace);

  const renameCollection = () => {
    if (collectionName == props.collection.name || nameExists) {
      console.error("We should not get here as we disable the button");
      return
    }
    let newWorkspace = structuredClone(workspace);
    let collection = newWorkspace.collections.find((collection: Collection) => {
      return collection.name === props.collection.name
    });
    if (!collection) {
      console.error("No collection found within the workspace with the given collection's name");
      // @TODO: display error
      return
    }
    collection.name = collectionName;
    backend.updateWorkspace(newWorkspace)
      .then(() => {
        updateWorkspaceStore(newWorkspace);
        props.onResolve(collectionName);
      }).catch(catchError);
  }

  const updateAndValidateGroupName = (name: string) => {
    let existsAlready = workspace.collections.some((collection: Collection) => { return collection.name == name });
    setNameExists(existsAlready);
    setCollectionName(name);
  }

  return (
    <Dialog header="Rename Collection" visible={props.isOpen} dismissableMask={false}
      style={{ width: '50vw' }}
      onHide={() => props.onResolve()}
      footer={
        <div>
          <Button label="Cancel" icon="pi pi-times" className={'p-button-secondary p-button-text'}
            onClick={() => props.onResolve()} />
          <Button disabled={nameExists || (collectionName === props.collection.name)} label="Rename" icon="pi pi-check"
            onClick={renameCollection}
            style={{ marginLeft: '80px' }} />
        </div>
      }>
      <div style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'flex-start',
        marginBottom: '50px',
        marginTop: '40px',
        flexGrow: 1
      }}>
        <div style={{ display: 'flex', width: '100%', marginBottom: '0px', alignItems: 'center' }}>
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'start' }}>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <h3>Name</h3>
              <InputText autoFocus={true} onChange={(e) => { updateAndValidateGroupName(e.target.value) }} onKeyPress={(event: any) => onCtrlEnter(event, renameCollection)} value={collectionName} style={{ marginLeft: '20px', flexBasis: '60%' }} />
            </div>

            <div style={{ marginTop: '30px', minHeight: '30px' }}>
              {
                nameExists &&
                <span className='p-error'>There exists already a collection with the same name. Choose another name.</span>
              }
            </div>
          </div>
        </div>
      </div>

    </Dialog >
  )
}
