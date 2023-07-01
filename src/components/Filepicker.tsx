import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";
import { catchError } from "../common/errorhandling";
import { backend } from "../rpc";
import { CopyToClipboard } from "./CopyToClipboard";

interface ComponentProps {
  path: string,
  updatePath: (newPath: string) => void,
  relativeBase?: string,
  style?: any
}


export function Filepicker(props: ComponentProps) {

  const chooseFile = () => {
    if (props.relativeBase) {
      backend.chooseFileRelativeTo(props.relativeBase).then((file: string) => {
        props.updatePath(file);
      }).catch(catchError);
    } else {
      backend.selectFile().then((file: string) => {
        props.updatePath(file);
      }).catch(catchError);
    }
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', ...(props.style ?? {}) }}>
      <label style={{ marginTop: '20px', flexBasis: '15%', textAlign: 'start' }}>Path (relative to request): </label>
      <div style={{ display: 'flex', alignItems: 'center', width: '100%', justifyContent: 'flex-start', marginTop: '10px' }}>
        <InputText disabled={true} style={{}} value={props.path} />
        {
          props.path !== "" && <CopyToClipboard value={props.path} />
        }
      </div>
      <Button style={{ marginTop: '20px' }} label={"Choose File"} onClick={chooseFile} />
    </div>
  )
}
