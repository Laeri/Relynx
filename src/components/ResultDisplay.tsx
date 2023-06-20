import { getHighlightContentType } from "../common/common";
import { dracula } from "react-syntax-highlighter/dist/cjs/styles/hljs";
import SyntaxHighlighter from "react-syntax-highlighter";
import { RequestResult } from '../bindings';
import { Tag } from "primereact/tag";
import { Button } from "primereact/button";
import { StatusCodeTag } from './StatusCodeTag';

interface ComponentProps {
  requestResult?: RequestResult,
  copyResultToClipboard: () => void
  clearResultText: () => void
}

export function ResultDisplay(props: ComponentProps) {

  if (!props.requestResult) {
    return (<></>);
  }

  const timeInMs = `${Math.floor(props.requestResult.total_time * 1000)} ms`

  return (
    props.requestResult && <div>
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
        <Button icon={"pi pi-copy"} onClick={props.copyResultToClipboard}
          tooltip={"Copy to clipboard"}
          className={"p-button-rounded p-button-text"} aria-label={"Copy to clipboard"}
          style={{ marginLeft: '10px' }}></Button>
        <Button onClick={props.clearResultText} icon="pi pi-times"
          tooltip={"Clear result"}
          className="p-button-rounded p-button-danger p-button-text" aria-label="Cancel"
          style={{ marginLeft: '10px' }} />
      </div>
      <SyntaxHighlighter contentEditable={true} className={"resultArea fade-in"}
        language={props.requestResult.content_type == null ? undefined : getHighlightContentType(props.requestResult.content_type)}
        style={dracula}>
        {props.requestResult.result}
      </SyntaxHighlighter>
    </div>

  )
}
