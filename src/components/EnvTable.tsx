import {
  DataTable
} from "primereact/datatable";
import { ColumnBodyOptions, Column, ColumnEditorOptions, ColumnEvent } from "primereact/column";
import { CSSProperties } from "react";
import { getUpdatedEnvironment, newEnvironmentSecret, newEnvironmentVariable } from "../model/environment";
import { InputText } from "primereact/inputtext";
import { useRequestModelStore } from "../stores/requestStore";
import { Environment, EnvironmentVariable, EnvironmentSecret } from '../bindings';
import { Checkbox, CheckboxChangeEvent } from "primereact/checkbox";
import { ActionDropdown } from "./ActionDropdown";
import { Button } from "primereact/button";

interface ComponentProps {
  updateEnvironmentInState: (name: string, newEnvironment: Environment) => void,
  isSecret: boolean,
  rowData: RowData[],
  onRemoveRowData: (rowData: RowData) => void
}

export interface RowData {
  index: number,
  name: string,
  initialValue: string,
  currentValue: string | undefined | null,
  description: string,
  persistToFile?: boolean,
  invalid?: boolean
  error?: string
}

export function EnvTable(props: ComponentProps) {

  const currentEnvironment = useRequestModelStore((state) => state.currentEnvironment)


  const nameTemplate = (rowData: RowData) => {
    return sharedTemplate(rowData.name, "Name", rowData.error, rowData.invalid)
  }

  const initialValueTemplate = (rowData: RowData) => {
    return sharedTemplate(rowData.initialValue, "Initial Value", undefined, false)
  }

  const currentValueTemplate = (rowData: RowData) => {
    return sharedTemplate(rowData.currentValue ?? "", "Current Value", undefined, false)
  }

  const descriptionTemplate = (rowData: RowData) => {
    return sharedTemplate(rowData.description ?? "", "Description", undefined, false)
  }

  const checkboxupdate = (rowIndex: number, checked: boolean) => {
    if (!currentEnvironment) {
      return
    }
    let newCurrentEnvironment = getUpdatedEnvironment(currentEnvironment, {});
    newCurrentEnvironment.secrets = currentEnvironment.secrets.map((secret: EnvironmentSecret, currentIndex: number) => {
      if (rowIndex === currentIndex) {
        let newSecret = newEnvironmentSecret();
        newSecret.name = secret.name;
        newSecret.initial_value = secret.initial_value;
        newSecret.current_value = secret.current_value;
        newSecret.persist_to_file = checked;
        return newSecret;
      } else {
        return secret;
      }
    });
    props.updateEnvironmentInState(newCurrentEnvironment.name, newCurrentEnvironment);
  }
  const checkboxTemplate = (initial: RowData) => {
    return (
      <Checkbox checked={initial.persistToFile || false}
        onChange={(event: CheckboxChangeEvent) => checkboxupdate(initial.index, event.checked || false)}
        style={{ height: '100%' }} />
    )
  }


  const onCellEditComplete = (e: { rowData: RowData; newValue: any; field: any; originalEvent: any; }) => {
    let { rowData, newValue, field, originalEvent: event } = e;

    // @ts-ignore
    rowData[field] = newValue ?? "";
    if (!currentEnvironment) {
      return
    }
    let newCurrentEnvironment = getUpdatedEnvironment(currentEnvironment, {})


    if (props.isSecret) {
      newCurrentEnvironment.secrets = currentEnvironment.secrets.map((secret: EnvironmentSecret, index: number) => {
        if (rowData.index === index) {
          let newSecret = newEnvironmentSecret()
          newSecret.name = rowData.name
          newSecret.initial_value = rowData.initialValue;
          if (rowData.currentValue && rowData.currentValue !== "") {
            newSecret.current_value = rowData.currentValue;
          } else {
            newSecret.current_value = null;
          }
          newSecret.persist_to_file = secret.persist_to_file // isn't edited here but by the checkbox...
          if (rowData.description && rowData.description !== "") {
            newSecret.description = rowData.description;
          } else {
            newSecret.description = null;
          }
          return newSecret
        } else {
          return secret
        }
      });
    } else {
      newCurrentEnvironment.variables = currentEnvironment.variables.map((variable: EnvironmentVariable, index: number) => {
        if (rowData.index === index) {

          let newVar = newEnvironmentVariable();
          newVar.name = rowData.name;
          newVar.initial_value = rowData.initialValue;
          if (rowData.currentValue && rowData.currentValue !== "") {
            newVar.current_value = rowData.currentValue;
          } else {
            newVar.current_value = null;
          }
          newVar.current_value = rowData.currentValue ?? null;
          if (rowData.description && rowData.description !== "") {
            newVar.description = rowData.description;

          } else {
            newVar.description = null;
          }
          return newVar;
        } else {
          return variable;
        }
      });
    }

    props.updateEnvironmentInState(newCurrentEnvironment.name, newCurrentEnvironment)
  }

  const textEditor = (options: ColumnEditorOptions, isName: boolean) => {
    if (!options.editorCallback) {
      return
    }
    let isValid = true;
    if (isName) {
      isValid = validateKeyName(options.value);
    }
    // no border otherwise row grows in size
    return <InputText type="text" style={isValid ? { border: 'none' } : {}} className={isValid ? '' : 'p-invalid'}
      value={options.value}
      onChange={(e) => options.editorCallback && options.editorCallback(e.target.value)} />;
  }


  const sharedTemplate = (value: string, placeholder: string, error?: string, invalid?: boolean) => {

    let placeholderStyle = {}

    if (value === '') {
      placeholderStyle = { color: 'rgba(255,255,255,255,0.2)' }
    }

    return <div style={{ display: 'flex', flexDirection: 'column' }}>
      <span className={invalid ? 'p-error' : ''}
        style={{ ...placeholderStyle }}>{(value !== '') ? value : placeholder}</span>
      {
        invalid &&
        <small className={'p-error'}
          style={{ marginTop: '5px', ...placeholderStyle }}>{error ? error : 'Invalid Name'}</small>
      }
    </div>

  }

  const actionButtonTemplate = (rowData: RowData, _columnBodyOptions: ColumnBodyOptions) => {
    return (
      <ActionDropdown styles={{ flexGrow: 1, marginRight: '3px' }}>
        <Button icon={'pi pi-trash'} className={'p-button p-button-text'}
          label={"Remove"}
          onClick={() => props.onRemoveRowData(rowData)} />
      </ActionDropdown>
    )
  }

  const validateKeyName = (e: ColumnEvent) => {
    // see if there is an element that already has the same name, in this case it is an invalid entry
    let noOtherElementWithSameName = !props.rowData.some((value: RowData, elementIndex: number) => {
      return value.name == e.newValue && e.rowData.index !== elementIndex
    })
    return noOtherElementWithSameName && e.newValue !== ''
  }

  const columnStyle: CSSProperties = { maxWidth: '10rem' }
  const columnStyleSmall: CSSProperties = { width: '4rem', textAlign: 'center' }
  const persistColumn: CSSProperties = { width: '10rem', textAlign: 'center' }
  return (
    <DataTable value={props.rowData} editMode="cell" className="editable-cells-table"
      style={{ width: '100%' }}
      responsiveLayout="scroll"
    >
      <Column field="name" filterPlaceholder={"Name"} header="Name"
        /*
                            cellEditValidator={validateKeyName}
        */
        editor={(options) => textEditor(options, true)}
        style={columnStyle}
        onCellEditComplete={onCellEditComplete} body={nameTemplate} />
      <Column field="initialValue" filterPlaceholder={"Initial Value"} header="Initial Value"
        editor={(options) => textEditor(options, false)} onCellEditComplete={onCellEditComplete}
        body={initialValueTemplate} style={columnStyle} />
      <Column field="currentValue" filterPlaceholder={"currentValue"} header={"Current Value"}
        editor={(options) => textEditor(options, false)}
        style={columnStyle}
        onCellEditComplete={onCellEditComplete} body={currentValueTemplate} />
      <Column field="description" filterPlaceholder={"description"} header={"Description"}
        editor={(options) => textEditor(options, false)}
        style={columnStyle}
        onCellEditComplete={onCellEditComplete} body={descriptionTemplate} />

      {
        props.isSecret ?
          <Column field="persistToFile" filterPlaceholder={"persistToFile"}
            headerTooltip={"If checked the secret will be saved in the file 'http-client.private.env' within your collection folder."}
            header="Persist To File"
            onCellEditComplete={onCellEditComplete} body={checkboxTemplate} style={persistColumn} />
          :
          <Column style={persistColumn} header={""} />

      }


      <Column header=""
        headerStyle={columnStyleSmall}
        bodyStyle={{ textAlign: 'center', overflow: 'visible' }}
        body={actionButtonTemplate} />
    </DataTable>
  )
}
