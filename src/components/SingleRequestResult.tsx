import { getHighlightContentType } from "../common/common";
import { dracula } from "react-syntax-highlighter/dist/cjs/styles/hljs";
import SyntaxHighlighter from "react-syntax-highlighter";
import { RequestResult } from '../bindings';
import { Tag } from "primereact/tag";
import { Button } from "primereact/button";
import { StatusCodeTag } from './StatusCodeTag';
import { CopyToClipboard } from "./CopyToClipboard";
import { backend } from "../rpc";
import { useMemo } from "react";

interface ComponentProps {
  requestResult: RequestResult,
  clearResult: (result: RequestResult) => void,
}

export function SingleRequestResult(props: ComponentProps) {
  if (!props.requestResult) {
    return <></>
  }

  const timeInMs = `${Math.floor(props.requestResult.total_time * 1000)} ms`

  const displaySingleRequestResult = useMemo(() => {
  }, [props.requestResult]);

  return (
    <div>
      <div
        style={{ display: 'flex', justifyContent: 'flex-end', flexGrow: 1, width: '100%', alignItems: 'center' }}>
        {props.requestResult && props.requestResult.status_code && <>
          <StatusCodeTag statusCode={props.requestResult.status_code} style={{ marginRight: '5px' }} />

          <Tag value={props.requestResult.content_type} style={{ maxHeight: '25px', marginRight: '5px' }} />

          <Tag value={timeInMs}
            className={".result-time"}
            style={{ maxHeight: '25px', marginRight: '5px', backgroundColor: 'lightgray' }} />

          <Tag value={`${props.requestResult.total_result_size}B`}
            style={{ maxHeight: '25px', marginRight: '10px', backgroundColor: 'lightgray' }} />
        </>}
        <CopyToClipboard value={props.requestResult.result} />
        <Button onClick={() => props.clearResult(props.requestResult)} icon="pi pi-trash"
          tooltip={"Clear Result"}
          className="p-button-rounded p-button-danger p-button-text" aria-label="Cancel"
          style={{ marginLeft: '10px' }} />
      </div>
      {props.requestResult.result_file &&
        <div style={{ marginTop: '10px' }}>
          Output saved in file: <Tag value={props.requestResult.result_file}></Tag>
          <CopyToClipboard value={props.requestResult.result_file} />
          <Button icon="pi pi-folder-open" className={"p-button-text"}
            tooltip={`Open folder: ${props.requestResult?.result_file_folder}`}
            style={{ width: '30px', height: '30px', marginLeft: '5px' }}
            onClick={() => backend.openFolderNative(props.requestResult?.result_file_folder ?? '')} />
        </div>
      }
      <SyntaxHighlighter contentEditable={true} className={"resultArea fade-in"}
        language={props.requestResult.content_type == null ? undefined : getHighlightContentType(props.requestResult.content_type)}
        style={dracula}>
        {props.requestResult.result}
      </SyntaxHighlighter>
    </div>
  )
}
