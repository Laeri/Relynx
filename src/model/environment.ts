import { Environment, EnvironmentVariable, EnvironmentSecret } from '../bindings';

export const environmentsToOptions = (environments: Environment[], withNone: boolean = false) => {
  let options = environments.map((environment: Environment) => {
    return { name: environment.name, value: environment.name };
  });
  if (withNone) {
    options = [...options, { name: "No Environment", value: '' }];
  }
  return options;
}

export const envDropdownStyle = {
}

export function getUpdatedEnvironment(environment: Environment, partial: Partial<Environment>): Environment {
  return { ...environment, ...partial };
}

export function newEnvironmentVariable(): EnvironmentVariable {
  let variable: EnvironmentVariable = {
    name: "",
    description: null,
    current_value: null,
    initial_value: ""
  };
  return variable;
}


export function newEnvironmentSecret(): EnvironmentSecret {
  let secret: EnvironmentSecret = {
    name: "",
    description: null,
    current_value: null,
    initial_value: "",
    persist_to_file: true
  };
  return secret;
}
