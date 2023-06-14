import { Dialog } from "primereact/dialog";
import { Button } from "primereact/button";
import { ImportResultComponent } from "../../components/ImportResultComponent";
import {ImportCollectionResult} from "../../bindings";

interface ComponentProps {
  isOpen: boolean
  onResolve: (result?: { collectionName: string, collectionPath: string, importCollectionFilepath: string }) => void
  onReject: () => void
  importCollectionResult: ImportCollectionResult
}

export function ImportResultModal(props: ComponentProps) {


  return (
    <Dialog header="Import Results" visible={props.isOpen} dismissableMask={false}
      style={{ width: '50vw' }}
      onHide={() => props.onResolve()}
      footer={
        <div>
          <Button label="Close" icon="pi pi-times" className={'p-button-secondary p-button-text'}
            onClick={() => props.onResolve()} />
        </div>
      }>
      <div style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'flex-start',
        marginBottom: '50px',
        marginTop: '40px'
      }}>
        {
          (props.importCollectionResult && props.importCollectionResult.collection) &&
          <ImportResultComponent collection={props.importCollectionResult.collection}
            onClearWarnings={undefined}
          />
        }
      </div>

    </Dialog>
  )
}
