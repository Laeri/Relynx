import { Dialog } from "primereact/dialog";
import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";
import { useState } from "react";


interface ComponentProps {
  isOpen: boolean
  onResolve: (keyValue: [string, string] | undefined) => void
  onReject: () => void,
}

export function AddCookieHeaderModal(props: ComponentProps) {

  const [key, setKey] = useState<string>("");
  const [value, setValue] = useState<string>("");

  const addCookieHeader = () => {
    props.onResolve([key, value]);
  }

  return (
    <Dialog header="Add Cookie" visible={props.isOpen} dismissableMask={false}
      style={{ width: '50vw' }}
      onHide={() => props.onResolve(undefined)}
      footer={
        <div>
          <Button label="Cancel" icon="pi pi-times" className={'p-button-secondary p-button-text'}
            onClick={() => props.onResolve(undefined)} />
          <Button disabled={key === ""} label="Add" icon="pi pi-plus"
            onClick={addCookieHeader}
            style={{ marginLeft: '80px' }} />
        </div>
      }>
      <div style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'flex-start',
        marginBottom: '50px',
        marginTop: '20px',
        flexGrow: 1
      }}>

        <p>Add a cookie which will be sent within the 'Cookie' header</p>
        <div style={{ display: 'flex', width: '100%', marginBottom: '0px', alignItems: 'center', marginTop: '20px' }}>
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'start',width: '100%' }}>
            <div style={{ display: 'flex', alignItems: 'center', width: '100%' }}>
              <h4 style={{ flexBasis: '10%' }}>Key</h4>
              <InputText autoFocus={true} value={key} onChange={(e) => setKey(e.target.value)}
                style={{ marginLeft: '20px', flexBasis: '60%' }} />
            </div>
            <div style={{ display: 'flex', alignItems: 'center', marginTop: '20px', width: '100%' }}>
              <h4 style={{ flexBasis: '10%' }}>Value</h4>
              <InputText value={value} onChange={(e) => setValue(e.target.value)}
                style={{ marginLeft: '20px', flexBasis: '60%' }} />
            </div>

          </div>
        </div>
      </div>
    </Dialog >
  )
}
