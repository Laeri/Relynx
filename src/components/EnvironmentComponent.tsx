import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";
import { Dropdown } from "primereact/dropdown";
import { Dialog } from "primereact/dialog";
import { useRequestModelStore } from "../stores/requestStore";
import { newEnvironment } from "../model/model";
import { CSSProperties, useContext, useEffect } from "react";
import { envDropdownStyle, environmentsToOptions, newEnvironmentSecret, newEnvironmentVariable } from "../model/environment";
import { ToastContext } from "../App";
import { catchError } from "../common/errorhandling";
import { create } from "react-modal-promise";
import { CreateEnvironmentModal } from "./modals/CreateEnvironmentModal";
import { EnvTable, RowData } from "./EnvTable";
import { EnvironmentSecret, EnvironmentVariable, Environment, Collection } from "../bindings";
import { scrollMainToTop } from "../common/common";
import { getUpdatedEnvironment } from "../model/request";
import { backend } from "../rpc";

interface ComponentProps {

}


export function EnvironmentComponent(_props: ComponentProps) {

  const environment = useRequestModelStore((state) => state.currentEnvironment);
  const environments = useRequestModelStore((state) => state.environments);
  const setEnvironments = useRequestModelStore((state) => state.setEnvironments);
  const setStoreCurrentEnvironment = useRequestModelStore((state) => state.setCurrentEnvironment);

  const collection = useRequestModelStore((state) => state.currentCollection as Collection);

  const toast = useContext(ToastContext)

  useEffect(() => {
    if (environment === undefined && environments.length > 0) {
      setStoreCurrentEnvironment(environments[0]);
    }
  }, [])

  const updateName = (name: string) => {
    if (!environment) {
      return
    }

    let oldName = environment.name;

    // TODO
    let newEnv = getUpdatedEnvironment(environment, { name: name });

    updateEnvironmentInState(oldName, newEnv);
  }


  const updateEnvironmentInState = (name: string, newEnvironment: Environment) => {
    let index = environments.findIndex((env: Environment) => env.name == name);
    let newEnvironments = [...environments];
    newEnvironments[index] = newEnvironment;
    setStoreCurrentEnvironment(newEnvironment)
    setEnvironments(newEnvironments)
    // @ts-ignore
    backend.saveEnvironments(collection, newEnvironments).then((_result: any) => {
      // ignored
    }).catch(catchError(toast))
  }


  const addEnvironmentVariable = () => {
    if (!environment) {
      return
    }
    let variables = [...environment.variables]
    let newVar = newEnvironmentVariable();
    variables.push(newVar)
    let newEnvironment = getUpdatedEnvironment(environment, { variables: variables })
    setStoreCurrentEnvironment(newEnvironment)
  }

  const addSecretVariable = () => {
    if (!environment) {
      return
    }
    let secrets = [...environment.secrets]
    let newSecret = newEnvironmentSecret();
    secrets.push(newSecret)
    let newEnvironment = getUpdatedEnvironment(environment, { secrets: secrets })
    setStoreCurrentEnvironment(newEnvironment)
  }


  const selectEnvironment = (name: any) => {
    let newCurrent = undefined
    if (name !== '') {
      newCurrent = environments.find((environment: Environment) => environment.name === name)
    }
    setStoreCurrentEnvironment(newCurrent)
  }

  const addEnvironment = (name: string) => {
    let newEnviron = newEnvironment({ name: name });
    let newEnvironments = [...environments, newEnviron]
    if (!collection) {
      return
    }
    backend.saveEnvironments(collection, newEnvironments).then(() => {
      setEnvironments(newEnvironments);
      setStoreCurrentEnvironment(newEnviron);
    }).catch(catchError(toast))
  }

  const openDeleteConfirmDialog = () => {
    let deleteDialogVisible = true;
    const deleteConfirmDialog = create(({ onResolve, onReject, isOpen }) => {
      return <Dialog header="Remove Environment" visible={deleteDialogVisible} style={{ width: '30vw', padding: 0 }}
        footer={
          <div>
            <Button label="No" icon="pi pi-times" onClick={() => onResolve()}
              className="p-button-text" />
            <Button label="Remove" icon="pi pi-check"
              onClick={() => onResolve(environment)}
              autoFocus />
          </div>
        } onHide={onReject}>
        <p>
          Are you sure you want to remove the environment <b>"{environment?.name}"</b>?
        </p>
        <br />
      </Dialog>
    })

    deleteConfirmDialog().then((currentEnvironment?: Environment) => {
      deleteDialogVisible = false;
      if (!currentEnvironment) {
        return
      }
      removeCurrentEnvironment();
    }).catch(() => {
    });

  }

  const removeCurrentEnvironment = () => {
    let currentEnvs = environments
    let newEnvironments = currentEnvs.filter((environment: Environment) => environment.name !== environment?.name)
    backend.saveEnvironments(collection, newEnvironments).then(() => {
      setEnvironments(newEnvironments)
      setStoreCurrentEnvironment(undefined)
    }).catch(catchError);
  }

  const openCreateEnvironmentDialog = () => {
    const addEnvironmentModal = create(({ isOpen, onResolve, onReject }) => {
      return <CreateEnvironmentModal isOpen={isOpen} onResolve={onResolve} onReject={onReject} />
    });

    addEnvironmentModal().then((environmentName?: string) => {
      if (!environmentName) {
        return
      }
      addEnvironment(environmentName)
    })
  }

  const getRowDataVariables = (): RowData[] => {
    if (!environment) {
      return []
    }
    let rowData = environment.variables.map((environmentVariable: EnvironmentVariable, index: number) => {
      return {
        index: index,
        name: environmentVariable.name,
        initialValue: environmentVariable.initial_value,
        currentValue: environmentVariable.current_value,
        description: environmentVariable.description ?? "",
        invalid: false
      } as RowData
    });
    return rowData
  }

  const getRowDataSecrets = (): RowData[] => {
    if (!environment) {
      return []
    }
    let rowData = environment.secrets.map((secret: EnvironmentSecret, index: number) => {
      return {
        index: index,
        name: secret.name,
        initialValue: secret.initial_value,
        currentValue: secret.current_value,
        description: secret.description ?? "",
        persistToFile: secret.persist_to_file,
        invalid: false
      } as RowData
    });
    return rowData
  }

  const invalidateDuplicateNames = (rowDataVars: RowData[], rowDataSecrets: RowData[]) => {
    let presentVarNames: { [name: string]: boolean } = {};

    // @TODO: show info that secrets should be stored below in the rowDataSecrets file
    rowDataVars.forEach((rowData: RowData) => {
      // empty keys are not validate
      if (rowData.name === '') {
        return
      }
      if (presentVarNames[rowData.name]) {
        rowData.invalid = true;
        rowData.error = "A variable with this name exists already!";
      } else {
        presentVarNames[rowData.name] = true;
      }
    });

    let presentSecretNames: { [name: string]: boolean } = {};


    rowDataSecrets.forEach((rowData: RowData) => {
      // empty keys are not validated
      if (rowData.name === '') {
        return
      }
      if (presentVarNames[rowData.name]) {
        rowData.invalid = true;
        rowData.error = "A variable with this name exists already!";
      } else if (presentSecretNames[rowData.name]) {
        rowData.invalid = true;
        rowData.error = "A secret with this name exists already!";
      } else {
        presentVarNames[rowData.name] = true;
      }
    });
  }


  const removeVariable = (rowData: RowData) => {
    if (!environment) {
      return
    }
    let variables = environment.variables.filter((_variable: EnvironmentVariable, index: number) => index !== rowData.index);
    let newEnvironment = getUpdatedEnvironment(environment, { variables: variables });
    updateEnvironmentInState(newEnvironment.name, newEnvironment);
  }

  const removeSecret = (rowData: RowData) => {
    if (!environment) {
      return
    }
    let secrets = environment.secrets.filter((_secret: EnvironmentVariable, index: number) => index !== rowData.index);
    let newEnvironment = getUpdatedEnvironment(environment, { secrets: secrets });
    updateEnvironmentInState(newEnvironment.name, newEnvironment);
  }

  const variableHelpText = "Variables can be inserted in your request. Use the syntax {{variableName}} within the request headers or params in order to define the placeholders for your variables. Variables are saved in the file 'http-client.env.json' for the respective environment in your collection folder. Please use secrets for passwords / tokens."
  const secretHelpText = "Secrets can contain values and be inserted into your request the same as variables. However, for secrets you can define if you want to persist them to a file within your collection or not. The secrets are stored unencrypted within the 'http-client.private.env.json' of your collection and be sure not to check this file into git and exclude it with .gitignore. Uncheck 'Persist to File' if you do not want the secret stored in the private environment file. You will need to enter the credentials again when using relynx."
  const helpTextStyle: CSSProperties = { marginTop: '10px', marginBottom: '20px', textAlign: 'left' }
  const envSectionStyle: CSSProperties = { display: 'flex', flexDirection: 'column' }

  const varRowData = getRowDataVariables()
  const secretRowData = getRowDataSecrets()
  invalidateDuplicateNames(varRowData, secretRowData)

  useEffect(() => {
    scrollMainToTop();
  }, [])

  return (
    <div className={'fade-in-fast'}
      style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', width: '100%' }}>

      <h1 style={{ marginTop: '20px', marginBottom: '50px' }}>Environment</h1>
      <p style={{ textAlign: 'left', marginBottom: '20px' }}>Create an environment that defines variables and secrets which can be inserted in your requests.</p>
      <div style={{ width: '100%', marginBottom: '200px' }}>
        <div style={{ display: 'flex', alignItems: 'center', marginBottom: '50px' }}>

          <Button icon={'pi pi-plus'} label={"Create Environment"} className={'p-button-raised'}
            onClick={openCreateEnvironmentDialog} />
          <Dropdown style={{ ...envDropdownStyle, marginLeft: '30px' }} optionLabel="name"
            value={environment?.name}
            options={environmentsToOptions(environments, true)}
            onChange={(e) => selectEnvironment(e.value)}
            placeholder={"No Environment"} />
        </div>

        {environment &&
          <>
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
              <h3>Name</h3>
              <div>
                <InputText autoFocus={true} value={environment.name} placeholder={"Name"}
                  onChange={(e) => updateName(e.target.value)} style={{ marginTop: '10px' }} />
                <Button label="Delete" onClick={openDeleteConfirmDialog} className={'p-button-outlined'}
                  icon="pi pi-trash" style={{ marginLeft: '50px' }} />
              </div>

            </div>

            <div style={{
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'flex-start',
              marginTop: '40px',
              marginBottom: '20px'
            }}>

              {/*Public Variables*/}
              <div style={envSectionStyle}>
                <h2 style={{ marginTop: '40px', marginBottom: '20px', textAlign: 'left' }}>Variables</h2>

                <p style={helpTextStyle}>
                  {variableHelpText}
                </p>

                <EnvTable isSecret={false} rowData={varRowData}
                  updateEnvironmentInState={updateEnvironmentInState}
                  onRemoveRowData={removeVariable} />

                <Button icon={'pi pi-plus'} label={"Add Variable"}
                  className={'p-button-raised p-button-text'}
                  style={{ marginTop: '30px', maxWidth: '180px' }} onClick={addEnvironmentVariable} />

              </div>

              {/*Secrets*/}
              <div style={envSectionStyle}>
                <h2 style={{ marginTop: '40px', marginBottom: '20px', textAlign: 'left' }}>Secrets</h2>

                <p style={helpTextStyle}>
                  {secretHelpText}
                </p>
                <EnvTable isSecret={true} rowData={secretRowData}
                  updateEnvironmentInState={updateEnvironmentInState}
                  onRemoveRowData={removeSecret} />

                <Button icon={'pi pi-plus'} label={"Add Secret"} className={'p-button-raised p-button-text'}
                  style={{ marginTop: '30px', maxWidth: '180px' }} onClick={addSecretVariable} />
              </div>
            </div>
          </>
        }
        {
          environments.length == 0 &&
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
            <p>This collection does not have any environments yet. Create a new one.</p>
          </div>
        }

      </div>
    </div>
  )
}
