import { Button } from "primereact/button";
import { Dropdown } from "primereact/dropdown";
import { InputText } from "primereact/inputtext";
import { InputTextarea } from "primereact/inputtextarea";
import { useEffect, useState } from "react";
import { RequestModel } from "../../bindings";
import { catchError } from "../../common/errorhandling";
import { DataSourceFromFilepath, DataSourceRaw, RequestBodyRaw } from "../../model/request"
import { backend } from "../../rpc";
import { RelynxState, useRequestModelStore } from "../../stores/requestStore";
import { CopyToClipboard } from "../CopyToClipboard";
import { RawType } from "./RequestBodyComp";

interface ComponentProps {
  bodyText: RequestBodyRaw,
  bodyFile: RequestBodyRaw,
  rawType: RawType
  updateBody: (newBody: RequestBodyRaw) => void,
  updateRawType: (newRawType: RawType) => void
}

export const RawTypes: { text: "text", file: "file" } = {
  text: "text",
  file: "file"
};

const optionText: { name: string, key: RawType } = { name: 'Raw Text', key: RawTypes.text };
const optionFromFile: { name: string, key: RawType } = { name: 'From File', key: RawTypes.file };


export function TextBody(props: ComponentProps) {

  const [text, setText] = useState<string>("");
  const [path, setPath] = useState<string>("");


  const currentRequest = useRequestModelStore((state: RelynxState) => state.currentRequest as RequestModel);

  useEffect(() => {
    setText((props.bodyText.Raw.data as DataSourceRaw<string>).Raw);
    setPath((props.bodyFile.Raw.data as DataSourceFromFilepath).FromFilepath);
  }, [props.bodyText, props.bodyFile]);

  const updateText = (newText: string) => {
    let newBody = structuredClone(props.bodyText);
    newBody.Raw.data = { Raw: newText };
    props.updateBody(newBody);
  }

  const updatePath = (newPath: string) => {
    let newBody = structuredClone(props.bodyFile);
    newBody.Raw.data = { FromFilepath: newPath };
    props.updateBody(newBody);
  }

  const chooseFile = () => {
    backend.chooseFileRelativeTo(currentRequest?.rest_file_path).then((file: string) => {
      updatePath(file);
    }).catch(catchError);
  }

  const updateRawType = (event: any) => {
    let newRawType = event.value.key;
    props.updateRawType(newRawType);
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
      <div style={{ display: 'flex', alignItems: 'center', marginBottom: '30px', marginTop: '20px' }}>
        <label>Source:</label>
        <div>{props.rawType}</div>
        <Dropdown onChange={updateRawType} style={{ marginLeft: '10px' }} optionLabel={"name"} options={[optionText, optionFromFile]} value={
          (props.rawType === "text") ? optionText : optionFromFile}
        />
      </div>
      {
        (props.rawType == "text") &&
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
          <h4 style={{ marginBottom: '20px' }}>Text</h4>
          <InputTextarea value={text} onChange={(e: any) => updateText(e.target.value)}
            rows={80}
            cols={100} autoResize={false} className={'json-body'} />
        </div>
      }
      {
        (props.rawType == "file") &&
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
          <h3>File</h3>

          <label style={{ marginTop: '20px', flexBasis: '15%', textAlign: 'start' }}>Path (relative to request): </label>
          <div style={{ display: 'flex', alignItems: 'center', width: '100%', justifyContent: 'flex-start', marginTop: '10px' }}>
            <InputText disabled={true} style={{}} value={path} />
            {
              path !== "" && <CopyToClipboard value={path} />
            }
          </div>
          <Button style={{ marginTop: '20px' }} label={"Choose File"} onClick={chooseFile} />
        </div>
      }
    </div>
  )
}
