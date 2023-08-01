import { Dropdown } from "primereact/dropdown";
import { InputTextarea } from "primereact/inputtextarea";
import { useEffect, useState } from "react";
import { RequestModel } from "../../bindings";
import { DataSourceFromFilepath, DataSourceRaw, RequestBodyRaw } from "../../model/request"
import { RelynxState, useRequestModelStore } from "../../stores/requestStore";
import { Filepicker } from "../Filepicker";
import { RawType } from "./RequestBodyComp";

interface ComponentProps {
  bodyText: RequestBodyRaw,
  bodyFile: RequestBodyRaw,
  rawType: RawType
  updateBody: (newBody: RequestBodyRaw) => void,
  updateRawType: (newRawType: RawType) => void,
  contentType?: string
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
  }, [props.bodyText.Raw.data]);

  useEffect(() => {
    setPath((props.bodyFile.Raw.data as DataSourceFromFilepath).FromFilepath);
  }, [props.bodyFile.Raw.data]);

  const updateText = (newText: string) => {
    setText(newText);
    let newBody = structuredClone(props.bodyText);
    newBody.Raw.data = { Raw: newText };
    props.updateBody(newBody);
  }

  const updatePath = (newPath: string) => {
    let newBody = structuredClone(props.bodyFile);
    newBody.Raw.data = { FromFilepath: newPath };
    props.updateBody(newBody);
  }

  const updateRawType = (event: any) => {
    let newRawType = event.value.key;
    props.updateRawType(newRawType);
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
      <div style={{ display: 'flex', alignItems: 'center', marginBottom: '30px', marginTop: '10px' }}>
        <label>Source:</label>
        <Dropdown onChange={updateRawType} style={{ marginLeft: '10px' }} optionLabel={"name"} options={[optionText, optionFromFile]} value={
          (props.rawType === "text") ? optionText : optionFromFile}
        />
      </div>
      {
        (props.rawType == "text") &&
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
          <h4 style={{ marginBottom: '20px' }}>Text</h4>
          <InputTextarea value={text} onChange={(e: any) => updateText(e.target.value)}
            style={{ width: '100%' }}
            rows={80}
            cols={70}
            autoResize={true} className={'json-body'} />
        </div>
      }
      {
        (props.rawType == "file") &&
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
          <h3>File</h3>
          <Filepicker path={path} updatePath={updatePath} relativeBase={currentRequest.rest_file_path} />
        </div>
      }
    </div>
  )
}
