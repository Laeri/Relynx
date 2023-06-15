import { InputText } from "primereact/inputtext";
import { Button } from "primereact/button";
import { backend } from '../rpc';
import { Dropdown } from "primereact/dropdown";
import { useContext, useEffect } from "react";
import { useRequestModelStore } from "../stores/requestStore";
import { catchError } from "../common/errorhandling";
import { ToastContext } from "../App";
import { useLocation, useNavigate } from "react-router";
import { Collection, Environment } from '../bindings';
import { environmentsToOptions, envDropdownStyle } from "../model/environment";
import { newWorkspace } from '../model/model';

export interface ComponentProps {
  collection: Collection
  displayPathTitle: boolean
}

export function CollectionInfo(props: ComponentProps) {
  const currentEnvironment = useRequestModelStore((state) => state.currentEnvironment);
  const setCurrentEnvironment = useRequestModelStore((state) => state.setCurrentEnvironment);
  const environments = useRequestModelStore((state) => state.environments);
  const hasEnvironments = environments.length > 0;
  const workspace = useRequestModelStore((state) => state.workspace);
  const toast = useContext(ToastContext);
  const navigate = useNavigate();
  const location = useLocation();

  const editEnvironments = () => {
    // if we are already in the environment view, then do not push state again
    if (location.pathname == '/collection/environment') {
      return;
    }
    let options = { replace: true };
    if (location.pathname == '/collection') {
      options.replace = false;
    }
    navigate('/collection/environment', options);
  }

  const selectEnvironment = (selected: string) => {
    let selectedEnvironment: Environment | undefined = undefined;
    if (selected !== '') {
      selectedEnvironment = environments.find((environment: Environment) => {
        return environment.name == selected
      });
    }

    let updatedWorkspace = newWorkspace(undefined)
    let newCollections = workspace.collections.map((aCollection: Collection) => {
      if (aCollection.path == props.collection.path) {
        props.collection.current_env_name = selectedEnvironment?.name ?? ''
        return props.collection
      } else {
        return aCollection
      }
    });
    updatedWorkspace.collections = newCollections;
    backend.updateWorkspace(updatedWorkspace).then(() => {
      setCurrentEnvironment(selectedEnvironment);
    }).catch(catchError(toast))
  }


  return (
    <div>

      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>

        <div style={{
          marginTop: '10px',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'flex-start',
          maxWidth: '100%'
        }}>
          {props.displayPathTitle && <span>Path</span>}
          <div style={{
            flexShrink: 1,
            minWidth: 0,
            maxWidth: '100%',
            display: 'flex',
            alignItems: 'center',
            marginTop: '10px',
            flexWrap: 'wrap'
          }}>
            <InputText disabled={false} value={props.collection.path}
              style={{ height: '30px', flexShrink: 1, minWidth: 0 }} />
            <Button icon="pi pi-folder-open" className={"p-button-text"}
              style={{ width: '30px', height: '30px', marginLeft: '5px' }}
              onClick={() => backend.openFolderNative(props.collection.path)} />
          </div>
        </div>


      </div>
      <div style={{ marginTop: '20px' }}>
        {
          hasEnvironments &&
          <>
            <div style={{ display: 'flex', alignItems: 'center', width: '100%', flexWrap: 'wrap' }}>
              <i className={"pi pi-box"} style={{ marginRight: '7px' }} />
              <Dropdown style={envDropdownStyle}
                tooltip={"Choose your environment for running requests"} optionLabel="name"
                value={currentEnvironment?.name}
                options={environmentsToOptions(environments, true)}
                onChange={(e) => selectEnvironment(e.value)} placeholder={"No Environment"} />
              <Button tooltip={"Edit Environments"} onClick={editEnvironments} icon={"pi pi-pencil"}
                className={"p-button-raised p-button-text"} style={{ minWidth: 0 }} />
            </div>
          </>
        }
        {
          !hasEnvironments &&
          <div style={{ display: 'flex', alignItems: 'center' }}>
            <Button onClick={editEnvironments} className={"p-button-raised p-button-text"}
              style={{ display: 'flex', alignItems: 'center' }}>
              <span className={"pi pi-box"} />
              <span className={""} style={{ marginLeft: '10px' }}>Environment</span>
              <span className={"pi pi-pencil"} style={{ marginLeft: '10px' }} />
            </Button>
          </div>
        }
      </div>
    </div>
  )
}
