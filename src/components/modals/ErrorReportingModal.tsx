import { Button } from "primereact/button"
import { Dialog } from "primereact/dialog"
import { InputText } from "primereact/inputtext"
import { useContext, useEffect, useState } from "react"
import { ToastContext } from "../../App"
import { RELYNX_MAIL } from "../../common/common"
import { backend } from "../../rpc"
import { RelynxState, useRequestModelStore } from "../../stores/requestStore"
import { CopyToClipboard } from "../CopyToClipboard"
import { Chip } from "primereact/chip";
import { getVersion } from '@tauri-apps/api/app';

interface ComponentProps {
  isOpen: boolean
  onResolve: (result?: {}) => void
  onReject: () => void,
  title: string,
  detail: string
}

export function ErrorReportingModal(props: ComponentProps) {

  const logPath = useRequestModelStore((state: RelynxState) => state.logPath);

  const toast = useContext(ToastContext);

  const [version, setVersion] = useState<string>("");

  useEffect(() => {
    getVersion().then((version: string) => {
      setVersion(version);
    });
  }, [])

  const copyLogfileContent = () => {
    backend.copy_logfile_content_to_clipboard().then(() => {
      toast.showSuccess("Copied log content to clipboard", "");
    });
  }
  return (
    <Dialog header="Report Errors" visible={props.isOpen} dismissableMask={false}
      onHide={props.onResolve}
      closable={false}
      modal={true}
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
        marginBottom: '0px',
        marginTop: '20px'
      }}>
        <h1>{props.title}</h1>
        <p>Version: v{version}</p>
        <p style={{ textAlign: 'left', marginTop: '20px' }}>{props.detail}</p>
        <p style={{ textAlign: 'left', marginTop: '20px' }}>
          If this error persists you can report it by writing an email to <Chip label={RELYNX_MAIL} /> <CopyToClipboard tooltip="Copy email to clipboard" value={RELYNX_MAIL} /> <br /><br />
          Please mention as well what you were doing before the error occurred and include relevant request files or the collection folder if possible.
        </p>
        {
          (logPath !== undefined) &&
          <div>
            <p style={{ marginTop: '20px', textAlign: 'left' }}>You can include the log file which you can find at the path.</p>
            <div style={{ display: 'flex', alignItems: 'center', marginTop: '20px' }}>
              <CopyToClipboard tooltip={"Copy log file path to clipboard"} value={logPath} />
              <InputText style={{ width: '100%' }} disabled={true} value={logPath} />
            </div>
            <Button style={{ marginTop: '20px' }} onClick={copyLogfileContent} outlined={true} label="Copy Log Content" />
            <p style={{ marginTop: '20px', textAlign: 'left' }}>Please check first before sending that the file does not include any information you do not want to share.</p>
          </div>
        }
      </div >
    </Dialog>
  )
}
