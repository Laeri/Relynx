import { Button } from "primereact/button"
import { InputText } from "primereact/inputtext"
import { useEffect, useState } from "react"
import { Environment, UrlEncodedParam } from "../../bindings"
import { RequestBodyUrlEncoded } from "../../model/request"
import { KeyValueRow } from "../KeyValueRow"

interface ComponentProps {
  body: RequestBodyUrlEncoded,
  updateBody: (newBody: RequestBodyUrlEncoded) => void,
  environment: Environment | undefined
}

export function UrlEncodedBody(props: ComponentProps) {

  const [urlEncodedStr, setUrlEncodedStr] = useState<string>();

  useEffect(() => {
    let urlEncoded = props.body.UrlEncoded.url_encoded_params.map((param: UrlEncodedParam) => {
      return encodeURIComponent(param.key) + "=" + encodeURIComponent(param.value)
    }).join("&");
    setUrlEncodedStr(urlEncoded);
  }, [props.body.UrlEncoded.url_encoded_params])

  const addParam = () => {
    let param = {
      key: "",
      value: ""
    }
    let newBody = structuredClone(props.body);
    newBody.UrlEncoded.url_encoded_params.push(param);
    props.updateBody(newBody);
  }

  const removeParam = (index: number) => {
    let newBody = structuredClone(props.body);
    newBody.UrlEncoded.url_encoded_params.splice(index, 1);
    props.updateBody(newBody);
  }


  const updateParamKey = (index: number, newKey: string) => {
    let param = props.body.UrlEncoded.url_encoded_params[index];
    let newParam: UrlEncodedParam = {
      key: newKey,
      value: param.value
    }
    let newBody = structuredClone(props.body);
    newBody.UrlEncoded.url_encoded_params[index] = newParam;
    props.updateBody(newBody);
  }

  const updateParamValue = (index: number, newValue: string) => {
    let param = props.body.UrlEncoded.url_encoded_params[index];
    let newParam: UrlEncodedParam = {
      key: param.key,
      value: newValue
    }
    let newBody = structuredClone(props.body);
    newBody.UrlEncoded.url_encoded_params[index] = newParam;
    props.updateBody(newBody);
  }

  return (
    <>
      {
        urlEncodedStr !== "" && <div style={{marginBottom: '20px', display: 'flex', alignItems: 'center', width: '100%'}}>
          <label>Body</label>
          <InputText style={{marginLeft: '20px', width: '80%'}} disabled={true} value={urlEncodedStr}/>
        </div>
      }
      {
        props.body.UrlEncoded.url_encoded_params.map((param: UrlEncodedParam, index: number) => {
          return <KeyValueRow key={index} keyProperty={param.key}
            valueProperty={param.value}
            active={true}
            keyLabel={"Param Name"} valueLabel={"Param Value"}
            updateKey={(key: string) => updateParamKey(index, key)}
            updateValue={(value: string) => updateParamValue(index, value)}
            updateActive={undefined}
            remove={() => removeParam(index)}
            style={{ marginTop: '20px' }}
            currentEnvironment={props.environment}
            withHeader={index == 0 ? { keyHeader: "Name", valueHeader: "Value" } : undefined}
          />
        })
      }
      <Button icon={'pi pi-plus'} label={"Param"} onClick={() => addParam()}
        className={"p-button-sm"}
        style={{ marginTop: '40px' }} />

    </>

  )
}
