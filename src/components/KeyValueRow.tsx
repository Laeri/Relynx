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
  updateActive: (active: boolean) => void
  remove: () => void
  style: any,
  currentEnvironment?: Environment
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
    props.updateActive(e.target.checked);
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
    if (event.key == 'Enter') {
      event.preventDefault();
      event.stopPropagation();
    }
  }
  const [suggestions, setSuggestions] = useState<any>([]);

  const itemTemplate = (suggestion: EnvironmentVariable | EnvironmentSecret) => {
    return (
      <div className="flex align-items-center" onClick={(_event: any) => handleItemSelected(suggestion)}>
        <span className="flex flex-column ml-2">
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
    <div style={{
      ...props.style,
      flexGrow: 1,
      width: '100%',
      display: 'flex',
      justifyContent: 'flex-start',
      alignItems: 'center'
    }}>
      <div style={{ flexGrow: 1, display: 'flex', maxWidth: '800px' }}>
        <InputText value={props.keyProperty} onChange={updateKey} placeholder={props.keyLabel}
          style={{ flexGrow: 1 }} />
        <Mention style={{ flexGrow: 1, marginLeft: '20px' }} suggestions={suggestions} onSearch={onSearch}
          field="Name" trigger={triggerKeySequence}
          value={props.valueProperty}
          onChange={updateValue}
          placeholder={props.valueLabel}
          onKeyDownCapture={handlePreventKeyEnterEvent}
          inputRef={inputRef}
          itemTemplate={itemTemplate} />
      </div>

      <div style={{ display: 'flex', alignItems: 'center', marginLeft: '30px' }}>
        <Checkbox onChange={updateActive} checked={props.active} style={{ height: '100%' }}
          title={"Active"}></Checkbox>
        <Button onClick={props.remove} icon="pi pi-times"
          className="p-button-rounded p-button-danger p-button-text" aria-label="Cancel"
          style={{ marginLeft: '10px' }} />
      </div>

    </div>
  )
}
