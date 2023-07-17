import { Button } from "primereact/button"
import { Environment, QueryParam } from "../bindings"
import { newQueryParam } from "../model/model";
import { KeyValueRow } from "./KeyValueRow"

interface ComponentProps {
  queryParams: QueryParam[],
  updateQueryParam: (oldQueryParam: QueryParam, newQueryParam: QueryParam) => void,
  removeQueryParam: (queryParam: QueryParam) => void,
  currentEnvironment?: Environment,
  addQueryParam: () => void
}

export function QueryParams(props: ComponentProps) {

  function updateQueryParamKey(queryParam: QueryParam, key: string) {
    let param: QueryParam = newQueryParam({ ...queryParam, key: key });
    props.updateQueryParam(queryParam, param);
  }

  function updateQueryParamValue(queryParam: QueryParam, value: string) {
    let param: QueryParam = newQueryParam({ ...queryParam, value: value });
    props.updateQueryParam(queryParam, param);
  }

  function updateQueryParamActive(queryParam: QueryParam, active: boolean) {
    let param = newQueryParam({ ...queryParam, active: active });
    props.updateQueryParam(queryParam, param);
  }

  return (
    <>
      {
        props.queryParams.map((queryParam: QueryParam, index: number) => {
          return <KeyValueRow key={index} keyProperty={queryParam.key}
            valueProperty={queryParam.value}
            active={queryParam.active}
            keyLabel={"Param Name"} valueLabel={"Param Value"}
            updateKey={(key: string) => updateQueryParamKey(queryParam, key)}
            updateValue={(value: string) => updateQueryParamValue(queryParam, value)}
            updateActive={(active: boolean) => updateQueryParamActive(queryParam, active)}
            remove={() => props.removeQueryParam(queryParam)}
            style={{ marginTop: '20px' }}
            currentEnvironment={props.currentEnvironment}
            withHeader={index == 0 ? { keyHeader: "Name", valueHeader: "Value" } : undefined}
          />
        })
      }
      <Button icon={'pi pi-plus'} label={"Query"} onClick={props.addQueryParam}
        className={"p-button-sm"}
        style={{ marginTop: '40px' }} />

    </>
  )
}
