import { InputText } from "primereact/inputtext";
import { Checkbox } from 'primereact/checkbox';
import { Button } from "primereact/button";
import { Ref, useRef, useState } from "react";
import { Mention } from "primereact/mention";
import { Environment, EnvironmentVariable, EnvironmentSecret } from '../bindings';

interface ComponentProps {
  keyProperty: string,
  valueProperty: string,
  keyLabel: string,
  valueLabel: string,
  active: boolean,
  updateKey: (key: string) => void,
  updateValue: (value: string) => void
  updateActive?: (active: boolean) => void
  remove: () => void
  style: any,
  currentEnvironment?: Environment,
  withHeader?: { keyHeader: string, valueHeader: string }
}

const triggerKeySequence = '{{';

export function KeyValueRow(props: ComponentProps) {

  // @ts-ignore
  const inputRef: Ref<HTMLInputElement> = useRef();

  const updateKey = (e: any) => {
    props.updateKey(e.target.value);
  }

  const updateValue = (e: any) => {
    props.updateValue(e.target.value);
  }

  const updateActive = (e: any) => {
    if (props.updateActive) {
      props.updateActive(e.target.checked);
    }
  }

  const handleItemSelected = (suggestion: EnvironmentVariable | EnvironmentSecret) => {
    // @ts-ignore
    let cursorPos = inputRef?.current?.selectionStart;
    let newValue = props.valueProperty.substring(0, cursorPos - triggerKeySequence.length)
      + "{{" + suggestion.name + "}}"
      + props.valueProperty.substring(cursorPos);
    props.updateValue(newValue);
  }

  // @TODO: for the moment prevent enter key, we will need to create a patch for the primereact library to fix this as
  // replace does not work with
  const handlePreventKeyEnterEvent = (event: any) => {

  }
  const [suggestions, setSuggestions] = useState<any>([]);

  const itemTemplate = (suggestion: EnvironmentVariable | EnvironmentSecret) => {
    return (
      <div className="flex align-items-center" >        <span className="flex flex-column ml-2">
        {suggestion.name}
      </span>
      </div>
    );
  }


  const onSearch = (_event: any) => {
    //in a real application, make a request to a remote url with the query and return suggestions, for demo we filter at client side
    setTimeout(() => {
      let suggestions;
      if (props.currentEnvironment) {
        suggestions = [
          ...props.currentEnvironment.variables.filter((variable: EnvironmentVariable) => variable.name !== ""),
          ...props.currentEnvironment.secrets.filter((secret: EnvironmentSecret) => secret.name !== "")
        ]
      }
      setSuggestions(suggestions);
    }, 250);
  }


  return (
    <div style={{ display: 'flex', flexDirection: 'column', width: '100%' }}>

      {props.withHeader !== undefined &&
        <div style={{ flexGrow: 1, display: 'flex', width: '100%', minWidth: '0', justifyContent: 'space-between' }}>
          <div style={{ width: '30%', textAlign: 'left' }}>Name</div>
          <div style={{ width: '40%', textAlign: 'left' }}>Value </div>
          {
            props.updateActive !== undefined &&
            <div style={{ width: '10%', display: 'flex', justifyContent: 'center', textAlign: 'center' }}>Active</div>
          }
          <div style={{ width: '10%' }}>Actions</div>
        </div>
      }

      <div style={{
        ...props.style,
        flexGrow: 1,
        width: '100%',
        display: 'flex',
        justifyContent: 'flex-start',
        alignItems: 'center',
      }}>

        <InputText value={props.keyProperty} onChange={updateKey} placeholder={props.keyLabel}
          style={{ minWidth: 0, width: '30%', height: '45px' }} />
        <Mention style={{ width: '40%', flexGrow: 1, marginLeft: '20px', minWidth: 0, height: '45px' }}
          suggestions={suggestions}
          onSearch={onSearch}
          field="Name" trigger={triggerKeySequence}
          value={props.valueProperty}
          onChange={updateValue}
          placeholder={props.valueLabel}
          inputRef={inputRef}
          itemTemplate={itemTemplate} />

        {
          props.updateActive !== undefined &&
          <Checkbox onChange={updateActive} checked={props.active} style={{ height: '100%', width: '10%' }}
            title={"Active"}></Checkbox>

        }
        <Button onClick={props.remove} icon="pi pi-times"
          className="p-button-rounded p-button-danger p-button-text" aria-label="Cancel"
          style={{ width: '10%' }} />
      </div>
    </div>
  )
}
