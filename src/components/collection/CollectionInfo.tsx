import { InputText } from "primereact/inputtext";
import { Button } from "primereact/button";
import { backend } from '../../rpc';
import { Dropdown } from "primereact/dropdown";
import { useRequestModelStore } from "../../stores/requestStore";
import { catchError } from "../../common/errorhandling";
import { routes } from "../../App";
import { useLocation, useNavigate } from "react-router";
import { Collection, Environment } from '../../bindings';
import { environmentsToOptions, envDropdownStyle } from "../../model/environment";
import { newWorkspace } from '../../model/model';

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
  const navigate = useNavigate();
  const location = useLocation();

  const editEnvironments = () => {
    // if we are already in the environment view, then do not push state again
    if (location.pathname == routes.environment) {
      return;
    }
    let options = { replace: true };
    if (location.pathname == routes.collection) {
      options.replace = false;
    }
    navigate(routes.environment, options);
  }

  const navigateToCookieJar = () => {
    // if we are already in the cookieJar view, then do not push state again
    if (location.pathname == routes.cookieJar) {
      return;
    }
    let options = { replace: true };
    if (location.pathname == routes.cookieJar) {
      options.replace = false;
    }
    navigate(routes.cookieJar, options);
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
    }).catch(catchError)
  }


  return (
    <div>

      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>

        <div style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'flex-start',
          maxWidth: '100%',
          marginTop: '10px'
        }}>
          {props.displayPathTitle && <h3>Path</h3>}
          <div style={{
            flexShrink: 1,
            minWidth: 0,
            maxWidth: '100%',
            display: 'flex',
            alignItems: 'center',
            marginTop: '10px',
          }}>
            <InputText disabled={false} value={props.collection.path}
              style={{ height: '30px', flexShrink: 1, minWidth: 0, direction: 'rtl' }} />
            <Button icon="pi pi-folder-open" className={"p-button-text"}
              tooltip={`Open folder: ${props.collection.path}`}
              style={{ width: '30px', height: '30px', marginLeft: '5px' }}
              onClick={() => backend.openFolderNative(props.collection.path)} />
          </div>
        </div>

        <Button onClick={navigateToCookieJar} style={{ marginTop: '20px' }} raised={true} text={true} icon={"pi pi-circle-off"} label="Cookie Jar" />


        <div style={{ marginTop: '10px', display: 'flex', flexDirection: 'column', alignItems: 'flex-start', width: '100%' }}>
          {props.displayPathTitle && <h3 style={{marginTop: '40px', marginBottom: '10px'}}>Environment</h3>}

          {
            hasEnvironments &&
            <div style={{ marginTop: '10px', width: '100%' }}>
              <div style={{ display: 'flex', alignItems: 'center', maxWidth: '100%' }}>
                <i className={"pi pi-box"} style={{ marginRight: '7px' }} />
                <Dropdown style={{ ...envDropdownStyle, minWidth: 0 }}
                  tooltip={currentEnvironment !== undefined ? currentEnvironment.name : "Choose your environment for running requests"} optionLabel="name"
                  value={currentEnvironment?.name}
                  options={environmentsToOptions(environments, true)}
                  onChange={(e) => selectEnvironment(e.value)} placeholder={"No Environment"}
                />
                <Button tooltip={"Edit Environments"} onClick={editEnvironments} icon={"pi pi-pencil"}
                  text={true} style={{ minWidth: 0, marginLeft: '5px' }} />
              </div>
            </div>
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
    </div>
  )
}
