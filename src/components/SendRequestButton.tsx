import { Button } from "primereact/button"

interface ComponentProps {
  isSendingRequest: boolean,
  doRequest: () => void,
  cancelRequest: () => void,
  disabled: boolean,
  overwriteLabel?: string,
  style: any
}

export function SendRequestButton(props: ComponentProps) {
  return (
    <div style={{ ...(props.style ?? {}), display: 'flex', alignItems: 'center' }}>
      <Button icon={props.isSendingRequest ? 'pi pi-spin pi-spinner' : 'pi pi-send'} label={props.isSendingRequest ? "Sending" : (props.overwriteLabel ?? "Send")} onClick={props.doRequest}
        style={{ }} disabled={props.disabled} />
      {
        props.isSendingRequest && <div style={{ display: 'flex', alignItems: 'center' }}>

          <Button onClick={props.cancelRequest} icon={"pi pi-times"}
            className="p-button-rounded p-button-danger p-button-text fade-in-fast" aria-label="Cancel Request"
            tooltip={"Cancel Request"}
            style={{ width: '10px', height: '10px', marginLeft: '10px' }} />
        </div>
      }
    </div>
  )
}
