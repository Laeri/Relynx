import { Dialog } from "primereact/dialog";
import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";
import { useState } from "react";
import { onCtrlEnter } from "../../common/eventhandling";
import { Collection, RequestModel } from "../../bindings";


interface ComponentProps {
  isOpen: boolean
  onResolve: (groupName?: string) => void
  onReject: () => void,
  collection: Collection,
  request: RequestModel,
  updateRequest: (newName: string) => Promise<void>
}

export function EditRequestNameModal(props: ComponentProps) {

  const [newRequestName, setNewRequestName] = useState<string>(props.request.name);
  const [sanitizedName, setSanitizedName] = useState<string | undefined>();
  const [nameError, setNameError] = useState<string | undefined>();

  const renameRequest = () => {
    console.log('start rename')
    props.updateRequest(newRequestName).then(() => {
      console.log('then RENAMED');
      props.onResolve();
    })
  }

  function updateRequestName(newName: string) {
    setNewRequestName(newName);
    validateRequestName(newName);
  }


  function validateRequestName(name: string): boolean {
    if (name === '') {
      setNameError("The name cannot be empty");
      return false;
    }
    setNameError(undefined);
    return true;
  }

  return (
    <Dialog header="Rename Request" visible={props.isOpen} dismissableMask={false}
      style={{ width: '50vw' }}
      onHide={props.onResolve}
      footer={
        <div>
          <Button label="Cancel" icon="pi pi-times" className={'p-button-secondary p-button-text'}
            onClick={() => props.onResolve()} />
          <Button disabled={nameError !== undefined || props.request.name === newRequestName} label="Rename" icon="pi pi-check"
            onClick={renameRequest}
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
              <InputText autoFocus={true} onKeyPress={(event: any) => onCtrlEnter(event, renameRequest)} value={newRequestName} onChange={(e) => updateRequestName(e.target.value)}
                style={{ marginLeft: '20px', flexBasis: '60%' }} />
            </div>

            {nameError !== '' &&
              <span className={"invalid mt-2"} style={{ textAlign: 'left' }}>{nameError}</span>
            }



            <div style={{ marginTop: '10px', minHeight: '50px' }}>
              {
                (sanitizedName !== undefined && sanitizedName !== newRequestName) &&
                <span className='p-error'>A valid request name is required. For example use '{sanitizedName}'.</span>
              }

            </div>
          </div>
        </div>
      </div>
    </Dialog >
  )
}
