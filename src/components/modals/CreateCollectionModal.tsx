import { Dialog } from "primereact/dialog";
import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";
import { useContext, useEffect, useState } from "react";
import { backend } from "../../rpc";
import { catchError } from "../../common/errorhandling";
import { ToastContext } from "../../App";
import { useRequestModelStore } from "../../stores/requestStore";
import { Collection } from "../../bindings";

interface ComponentProps {
  isOpen: boolean
  onResolve: (result?: { collectionName: string, collectionPath: string }) => void
  onReject: () => void
}

export function CreateCollectionModal(props: ComponentProps) {

  const [collectionName, setCollectionName] = useState<string>("");
  const [collectionPath, setCollectionPath] = useState<string>("");

  const collections = useRequestModelStore((state) => state.workspace.collections)

  const [nameError, setNameError] = useState<string>("")

  const toast = useContext(ToastContext);

  const [isDirEmpty, setIsDirEmpty] = useState<boolean>(true);

  useEffect(() => {
    let defaultName = "New Collection"
    setCollectionName(defaultName);
    validateName(defaultName);

  }, []);

  const openCollectionDirectoryPicker = () => {
    backend.selectDirectory((result: string) => {
      setCollectionPath(result);
    });
  }

  const updateCollectionName = (newName: string) => {
    setCollectionName(newName);
    validateName(newName);
  }

  const validateName = (newName: string) => {

    let nameExistsAlready = collections.some((collection: Collection) => collection.name == newName)

    if (nameExistsAlready) {
      setNameError("There exists already a collection with the same name! Choose another one");
    } else {
      setNameError("");
    }

    if (newName == "") {
      setNameError("Name cannot be empty")
    }
  }


  return (
    <Dialog header="Create Collection" visible={props.isOpen} dismissableMask={false}
      style={{ width: '50vw' }}
      onHide={props.onResolve}
      modal={true}
      footer={
        <div>
          <Button label="Cancel" icon="pi pi-times" className={'p-button-secondary p-button-text'}
            onClick={() => props.onResolve()} />
          <Button label="Create" icon="pi pi-check"
            onClick={() => props.onResolve({ collectionName, collectionPath })}
            style={{ marginLeft: '80px' }} />
        </div>
      }>
      <div style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'flex-start',
        marginBottom: '50px',
        marginTop: '40px'
      }}>

        <div style={{ display: 'flex', width: '100%', flexDirection: "column", alignItems: 'flex-start', minHeight: '100px' }}>
          <div style={{ display: 'flex', width: '100%', alignItems: 'center' }}>
            <h3 style={{}}>Name</h3>

            <div style={{ display: "flex", flexDirection: "column", marginLeft: '20px', flexBasis: '60%' }}>
              <InputText autoFocus={true} value={collectionName}
                onChange={(e) => updateCollectionName(e.target.value)}
              />

            </div>
          </div>
          {nameError !== '' &&
            <span className={"invalid mt-2"} style={{ textAlign: 'left' }}>{nameError}</span>

          }
        </div>
        <p style={{ textAlign: 'left' }}>Choose an folder for your new collection. Requests will be stored as files and each group will have its own subfolder.</p>
        <h3 style={{ marginTop: '30px', marginBottom: '20px' }}>Folder</h3>
        <div style={{ display: 'flex', width: '100%' }}>
          <Button label={"Choose Folder"}
            onClick={openCollectionDirectoryPicker} style={{}} />
          <div style={{ display: 'flex', flexDirection: 'column' }}>
            <InputText value={collectionPath} style={{ flexGrow: 1, marginLeft: '20px' }} disabled={true} />

          </div>

        </div>
      </div>

    </Dialog >
  )
}
