import { Dialog } from "primereact/dialog";
import { Button } from "primereact/button";
import { useContext, useState } from "react";
import { ToastContext } from "../../App";
import { RadioButton } from "primereact/radiobutton";
import { InputText } from "primereact/inputtext";
import { RelynxState, useRequestModelStore } from "../../stores/requestStore";
import { Collection } from "../../bindings";

interface ComponentProps {
  isOpen: boolean
  onResolve: (result?: { importType: ImportType, collectionName: string}) => void
  onReject: () => void
}

export enum ImportType {
  Postman,
  JetbrainsHttpRest
}

export function ImportCollectionModal(props: ComponentProps) {

  const [importType, setImportType] = useState<ImportType>(ImportType.Postman);
  const [collectionName, setCollectionName] = useState<string>("");
  const [nameExists, setNameExists] = useState<boolean>(false);
  const workspace = useRequestModelStore((state: RelynxState) => state.workspace);


  const updateCollectionName = (newName: string) => {
    setCollectionName(newName);
    let nameExists = workspace.collections.some((col: Collection) => col.name === newName);
    console.log('name exists', workspace.collections);
    console.log('new name: ', newName);
    setNameExists(nameExists);
  }

  return (
    <Dialog header="Import Collection" visible={props.isOpen} dismissableMask={false}
      style={{ width: '50vw' }}
      onHide={() => props.onResolve()}
      footer={
        <div>
          <Button label="Cancel" icon="pi pi-times" className={'p-button-secondary p-button-text'}
            onClick={() => props.onResolve()} />
          <Button disabled={collectionName === "" || nameExists} label="Continue" icon="pi pi-check" onClick={() => props.onResolve({
            importType: importType, collectionName: collectionName
          })}
            style={{ marginLeft: '80px' }} />
        </div>
      }>
      <div style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'flex-start',
        marginBottom: '0px',
        marginTop: '0px'
      }}>
        <h3 style={{ marginTop: '30px', marginBottom: '20px' }}>Import type</h3>
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
          <div style={{ display: 'flex', alignItems: 'center' }}>
            <RadioButton inputId="postman" name="postman" value={ImportType.Postman} onChange={(e: any) => setImportType(e.value)} checked={importType === ImportType.Postman} />
            <label htmlFor="postman" style={{ marginLeft: '20px' }} className="ml-2">Postman Collection (v2.1.0)</label>
          </div>
          <div style={{ marginTop: '30px', display: 'flex', alignItems: 'center' }}>
            <RadioButton inputId="jetbrains-http-rest" name="postman" value={ImportType.JetbrainsHttpRest} onChange={(e: any) => setImportType(e.value)} checked={importType === ImportType.JetbrainsHttpRest} />
            <label htmlFor="jetbrains-http-rest" style={{ marginLeft: '20px' }} >Jetbrains Folder (containing .http or .rest files)</label>
          </div>
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', marginTop: '40px' }}>
          <h3>Collection Name</h3>
          <p style={{ marginTop: '10px' }}>Choose a name for your import collection</p>
          <InputText style={{ marginTop: '10px' }} value={collectionName} onChange={(e: any) => { updateCollectionName(e.target.value) }} />
        </div>
        <div style={{marginTop: '10px', minHeight: '30px'}}>
          {nameExists && <p className="p-error">A collection with the same name exists already</p>}
        </div>
      </div>

    </Dialog>
  )
}
