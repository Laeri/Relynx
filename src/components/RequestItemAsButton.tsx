import { Button } from "primereact/button";
import { useRef } from "react";
import { Tag } from "primereact/tag";
import { Message } from "primereact/message";
import { OverlayPanel } from "primereact/overlaypanel";
import { hasInvalidFileBody } from "../common/requestUtils";
import { RequestModel, ImportWarning } from '../bindings';
import { requestMethodToString } from "../model/request";

interface ComponentProps {
  requestId: string,
  onClick: () => void,
  highlighted: boolean,
  requestModel: RequestModel,
  label: string,

  importWarnings: ImportWarning[]
}

export function RequestItemAsButton(props: ComponentProps) {
  const invalidBodyWarnMessage = useRef<OverlayPanel>(null);
  const importWarnMessage = useRef<OverlayPanel>(null);
  return (
    <Button onClick={props.onClick}
      className={"p-button-raised p-button-text" + (props.highlighted ? 'p-button-secondary' : '')}
      style={{ display: 'flex', flexGrow: 1, paddingLeft: '3px', paddingRight: '3px' }}
      key={props.requestId}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%' }}>
        <Tag value={requestMethodToString(props.requestModel.method)} />
        <span style={{ flexGrow: 1 }}>{props.label}</span>
        {props.importWarnings.length > 0 &&
          <>
            <div
              style={{ padding: '3px', display: 'flex', justifyContent: 'center', alignItems: 'center' }}
              onMouseEnter={(e) => importWarnMessage?.current?.show(e, e.target)}

              onMouseLeave={(e) => importWarnMessage?.current?.hide()}
            ><i className="pi pi-exclamation-triangle color-warn" /></div>

            <OverlayPanel ref={importWarnMessage}>
              <Message severity={"warn"}
                text={"There were problems during the import with this request!"} />
            </OverlayPanel>
          </>
        }
        {hasInvalidFileBody(props.requestModel) &&
          <>
            <div
              style={{ padding: '3px', display: 'flex', justifyContent: 'center', alignItems: 'center' }}
              onMouseEnter={(e) => invalidBodyWarnMessage?.current?.show(e, e.target)}

              onMouseLeave={(e) => invalidBodyWarnMessage?.current?.hide()}
            ><i className="pi pi-exclamation-triangle color-warn" /></div>

            <OverlayPanel ref={invalidBodyWarnMessage}>
              <Message severity={"warn"}
                text={"This request has a file without valid file path as body type set!"} />
            </OverlayPanel>
          </>
        }
      </div>

    </Button >
  )
}
