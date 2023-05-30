import { Dialog } from "primereact/dialog";
import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";
import { useContext, useState } from "react";
import { backend } from "../../rpc";
import { catchError } from "../../common/errorhandling";
import { ToastContext } from "../../App";

interface ComponentProps {
  isOpen: boolean
  onResolve: (result?: { collectionPath: string }) => void
  onReject: () => void,
  collectionName?: string
}

export function AddCollectionModal(props: ComponentProps) {

  const [collectionPath, setCollectionPath] = useState<string>(props.collectionName ?? "");

  const toast = useContext(ToastContext);

  const openCollectionDirectoryPicker = () => {
    backend.selectDirectory()
      .then((result: string) => {
        setCollectionPath(result);
      })
      .catch(catchError(toast));
  }

  return (
    <Dialog header="Add Collection" visible={props.isOpen} dismissableMask={false}
      style={{ width: '50vw' }}
      onHide={props.onResolve}
      footer={
        <div>
          <Button label="Cancel" icon="pi pi-times" className={'p-button-secondary p-button-text'}
            onClick={() => props.onResolve()} />
          <Button label="Add" icon="pi pi-check"
            onClick={() => props.onResolve({ collectionPath })}
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
        <h3 style={{ marginTop: '30px', marginBottom: '20px' }}>Folder</h3>
        <div style={{ display: 'flex', width: '100%' }}>
          <Button label={"Open"}
            onClick={openCollectionDirectoryPicker} style={{}} />
          <InputText value={collectionPath} style={{ flexGrow: 1, marginLeft: '20px' }} disabled={true} />
        </div>
      </div>

    </Dialog>
  )
}
