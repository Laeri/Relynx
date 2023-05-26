import { EnvVarDescription, Environment } from '../bindings';

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
  maxWidth: '150px'
}


export function NewEnvVarDescription(name: string, isSecret: boolean, description: string): EnvVarDescription {
  return {
    env_var_name: name,
    description: description,
    is_secret: isSecret,
  }
}
