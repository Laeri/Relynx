import { Dialog } from "primereact/dialog";
import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";
import { useContext, useState } from "react";
import { backend } from "../../rpc";
import { ToastContext } from "../../App";
import { catchError } from "../../common/errorhandling";
import { Message } from "primereact/message";
import { ScrollPanel } from 'primereact/scrollpanel';

interface ComponentProps {
  isOpen: boolean
  onResolve: (result?: { collectionName: string, collectionPath: string, importCollectionFilepath: string }) => void
  onReject: () => void
}

export function ImportPostmanModal(props: ComponentProps) {

  const [collectionName, _setCollectionName] = useState<string>("");
  const [collectionPath, setCollectionPath] = useState<string>("");
  const [importCollectionFilepath, setImportCollectionFilepath] = useState<string>("");

  const toast = useContext(ToastContext);

  const importCollectionFilePicker = () => {
    backend.selectFile((result: string) => {
      setImportCollectionFilepath(result);
    })
  }

  const openCollectionDirectoryPicker = () => {
    backend.selectDirectory((result: string) => {
      setCollectionPath(result);
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
          <Button label="Import" icon="pi pi-check" onClick={() => props.onResolve({
            collectionName,
            collectionPath,
            importCollectionFilepath
          })}
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
        <h3 style={{ marginTop: '30px', marginBottom: '20px' }}>Postman Collection File</h3>
        <p style={{}}>Choose the postman collection json file that you want to import</p>
        <div style={{ display: 'flex', width: '100%', marginTop: '10px' }}>
          <Button label={"Choose"}
            onClick={importCollectionFilePicker} style={{}} />
          <InputText autoFocus={true} value={importCollectionFilepath} style={{ flexGrow: 1, marginLeft: '20px' }}
            disabled={true} />
        </div>


        <h3 style={{ marginTop: '30px', marginBottom: '20px' }}>Target Location</h3>
        <p style={{}}>Where do you want to store the imported collection?</p>
        <div style={{ marginTop: '10px', display: 'flex', width: '100%' }}>
          <Button label={"Choose Empty Folder"}
            onClick={openCollectionDirectoryPicker} style={{}} />
          <InputText value={collectionPath} style={{ flexGrow: 1, marginLeft: '20px' }} disabled={true} />
        </div>


        <ScrollPanel style={{ marginTop: '30px', width: '100%', height: '150px' }}>
          <Message severity="warn" text="Postman Import Caveats" />
          <div>

            <ul style={{ marginTop: '20px', textAlign: 'left' }}>
              <li style={{}}>
                <p>
                  Postman requests have hidden headers that are not visible in their UI but will be sent with the request.
                  For example, body content length, the user-agent, content type, etc.
                  These will not be included in the import and have to be added manually if they are required for your request.
                </p>
              </li>

              <li style={{ marginTop: '10px' }}>
                <p>
                  If your request uses files as input you might want to check that they were included in the import correctly.
                  In case a request file is missing the request itself will have a warning symbol.
                </p>
              </li>
            </ul>
          </div>
        </ScrollPanel>

      </div>

    </Dialog>
  )
}
