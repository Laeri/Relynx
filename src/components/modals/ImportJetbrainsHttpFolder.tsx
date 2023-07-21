import { Dialog } from "primereact/dialog";
import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";
import { useContext, useState } from "react";
import { backend } from "../../rpc";
import { ToastContext } from "../../App";
import { catchError } from "../../common/errorhandling";

interface ComponentProps {
  isOpen: boolean
  onResolve: (result?: { collectionPath: string, collectionName: string }) => void
  onReject: () => void
}

export function ImportJetbrainsHttpFolder(props: ComponentProps) {

  const [collectionName, _setCollectionName] = useState<string>("");
  const [jetbrainsFolder, setJetbrainsFolder] = useState<string>("");

  const toast = useContext(ToastContext);

  const openCollectionDirectoryPicker = () => {
    backend.selectDirectory((result: string) => {
        setJetbrainsFolder(result);
      });
  }

  return (
    <Dialog header="Import Collection" visible={props.isOpen} dismissableMask={false}
      style={{ width: '50vw' }}
      onHide={() => props.onResolve()}
      footer={
        <div>
          <Button label="Cancel" icon="pi pi-times" className={'p-button-secondary p-button-text'}
            onClick={() => props.onResolve()} />
          <Button label="Import" icon="pi pi-check" onClick={() => {
            props.onResolve({
              collectionPath: jetbrainsFolder,
              collectionName: collectionName
            })
          }}
            style={{ marginLeft: '80px' }} />
        </div>
      }>
      <div style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'flex-start',
        marginBottom: '50px',
        marginTop: '0px'
      }}>
        <h3 style={{ marginTop: '30px', marginBottom: '20px' }}>Jetbrains Collection Folder (.http, .rest)</h3>

        <p style={{}}>Which collection do you want to import?
          All .http and .rest files will be available as requests and all folders will be shown as groups in the tree hierarchy.
        </p>
        <div style={{ marginTop: '30px', display: 'flex', width: '100%' }}>
          <Button label={"Select Import Folder"}
            onClick={openCollectionDirectoryPicker} style={{}} />
          <InputText value={jetbrainsFolder} style={{ flexGrow: 1, marginLeft: '20px' }} disabled={true} />
        </div>
      </div>

    </Dialog>
  )
}
