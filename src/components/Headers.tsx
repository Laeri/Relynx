import { Button } from "primereact/button";
import { Header, RequestModel } from "../bindings"
import { openAddCookieModal } from "../common/modal";
import { newRequestHeader, updatedRequestModel } from "../model/model";
import { RelynxState, useRequestModelStore } from "../stores/requestStore";
import { KeyValueRow } from "./KeyValueRow"


interface ComponentProps {
  request: RequestModel,
  updateRequest: (newRequest: RequestModel) => void
}

export function Headers(props: ComponentProps) {

  const currentEnvironment = useRequestModelStore((state: RelynxState) => state.currentEnvironment);

  function addHeader() {
    let newRequestModel: RequestModel = updatedRequestModel(
      props.request, {
      headers: [...props.request.headers, newRequestHeader(undefined)]
    });
    props.updateRequest(newRequestModel);
  }

  function addCookie() {
    openAddCookieModal().then((keyValue: [string, string] | undefined) => {
      if (!keyValue) {
        return
      }

      let headers = [...props.request.headers];
      let cookieIndex = headers.findIndex((header: Header) => header.key.toLowerCase() === 'cookie');

      if (cookieIndex !== -1) {
        let cookieHeader = headers[cookieIndex];
        cookieHeader.value += `;${keyValue[0]}=${keyValue[1]}`
      } else {
        let cookieHeader: Header = { key: 'Cookie', value: `${keyValue[0]}=${keyValue[1]}`, active: true };
        headers.push(cookieHeader);
      }

      let newRequestModel: RequestModel = updatedRequestModel(
        props.request, {
        headers: headers
      });
      props.updateRequest(newRequestModel);
    })
  }

  function removeHeader(headerKey: string) {
    let newRequestHeaders = props.request.headers.filter((current: Header) => current.key.toLowerCase() !== headerKey.toLowerCase())
    let newRequestModel: RequestModel = updatedRequestModel(
      props.request, {
      headers: newRequestHeaders
    });
    return props.updateRequest(newRequestModel);
  }

  function updateHeaderKey(requestHeader: Header, key: string) {
    let header = newRequestHeader({ ...requestHeader, key: key });
    updateRequestHeader(requestHeader, header);
  }

  function updateHeaderValue(requestHeader: Header, value: string) {
    let header = newRequestHeader({ ...requestHeader, value: value });
    updateRequestHeader(requestHeader, header);
  }

  function updateHeaderActive(requestHeader: Header, active: boolean) {
    let header = newRequestHeader({ ...requestHeader, active: active });
    updateRequestHeader(requestHeader, header);
  }

  function updateRequestHeader(oldHeader: Header, newHeader: Header) {
    let newRequestHeaders = [...props.request.headers];
    let index = newRequestHeaders.indexOf(oldHeader);
    newRequestHeaders[index] = newHeader;

    let newRequestModel = updatedRequestModel(props.request, { headers: newRequestHeaders });
    props.updateRequest(newRequestModel);
  }

  return (
    <div className="headers-block"
      style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
      <h2 style={{ marginBottom: '20px' }}>Headers</h2>
      {
        props.request.headers?.map((header: Header, index: number) => {
          return <KeyValueRow key={index} keyProperty={header.key}
            valueProperty={header.value}
            active={header.active}
            keyLabel={"Header Name"} valueLabel={"Header Value"}
            updateKey={(key: string) => updateHeaderKey(header, key)}
            updateValue={(value: string) => updateHeaderValue(header, value)}
            updateActive={(active: boolean) => updateHeaderActive(header, active)}
            remove={() => removeHeader(header.key)}
            style={{ marginTop: '20px' }}
            currentEnvironment={currentEnvironment}
            withHeader={index == 0 ? { keyHeader: 'Name', valueHeader: "Value" } : undefined}
          />
        })
      }
      <div style={{ display: 'flex', alignItems: 'center' }}>
        <Button icon={'pi pi-plus'} label={"Header"} onClick={addHeader}
          className={"p-button-sm"}
          style={{ margin: '40px 0px' }} />

        <Button icon={'pi pi-plus'} label={"Cookie"} onClick={addCookie}
          className={"p-button-sm"}
          style={{ marginLeft: '40px' }} />
      </div>
    </div>

  )
}
