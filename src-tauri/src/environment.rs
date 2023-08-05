use std::{collections::HashMap, path::PathBuf};

use crate::{
    config::{load_collection_config, save_collection_config},
    error::RelynxError,
    model::{
        Collection, EnvVarDescriptions, Environment, EnvironmentSecret, EnvironmentVariable,
        SingleEnvVarDescription,
    },
};
pub const HTTP_ENV_FILENAME: &str = "http-client.env.json";
pub const PRIVATE_HTTP_ENV_FILENAME: &str = "http-client.private.env.json";

// values could be anything in the json, but when we load the environment all variables are strings
pub type EnvKeyValues = HashMap<String, String>;
pub type EnvFileStructure = HashMap<String, EnvKeyValues>;

pub fn load_environments_from_files(
    env_file_path: Option<&std::path::PathBuf>,
    private_env_file_path: Option<&std::path::PathBuf>,
    collection_path: Option<&std::path::PathBuf>,
) -> Result<Vec<Environment>, RelynxError> {
    let mut environments: HashMap<String, Environment> = HashMap::new();

    if let Some(env_file_path) = env_file_path {
        let env_structure = load_env_structure(env_file_path)?;

        for (env_name, key_val_map) in env_structure.iter() {
            let variables: Vec<EnvironmentVariable> = key_val_map
                .iter()
                .map(|(var_name, value)| EnvironmentVariable {
                    name: var_name.clone(),
                    initial_value: value.clone(),
                    current_value: None,
                    description: None, // @TODO: get description from collectionconfig :(
                })
                .collect();
            let environment = Environment {
                name: env_name.clone(),
                variables,
                secrets: vec![],
            };
            environments.insert(env_name.to_string(), environment);
        }
    }
    if let Some(private_env_file_path) = private_env_file_path {
        let private_env_structure = load_env_structure(private_env_file_path)?;

        for (env_name, key_val_map) in private_env_structure.into_iter() {
            let environment = environments
                .entry(env_name.clone())
                .or_insert(Environment::new(env_name.clone()));

            let secrets: Vec<EnvironmentSecret> = key_val_map
                .into_iter()
                .map(|(var_name, value)| EnvironmentSecret {
                    name: var_name,
                    initial_value: value,
                    current_value: None,
                    description: None, // @TODO: get description from collectionconfig :(
                    persist_to_file: true,
                })
                .collect();
            environment.secrets = secrets;
        }
    }

    if let Some(collection_path) = collection_path {
        if let Ok(mut collection_config) = load_collection_config(collection_path) {
            let mut to_remove_env: Vec<String> = Vec::new();
            for (env_name, descriptions) in collection_config.env_var_descriptions.iter_mut() {
                let environment = environments.get_mut(env_name);
                if environment.is_none() {
                    to_remove_env.push(env_name.clone());
                    continue;
                }
                let environment = environment.unwrap();
                let mut to_remove_descr: Vec<usize> = Vec::new();
                for (index, description) in descriptions.iter().enumerate() {
                    if description.is_secret {
                        match environment
                            .variables
                            .iter_mut()
                            .find(|var| var.name == description.env_var_name)
                        {
                            Some(variable) => {
                                variable.description = Some(description.description.clone());
                            }
                            None => to_remove_descr.push(index),
                        };
                    } else {
                        match environment
                            .secrets
                            .iter_mut()
                            .find(|var| var.name == description.env_var_name)
                        {
                            Some(secret) => {
                                secret.description = Some(description.description.clone());
                            }
                            None => to_remove_descr.push(index),
                        };
                    }
                }
                for description_index in to_remove_descr {
                    descriptions.remove(description_index);
                }
            }

            for env_name in to_remove_env {
                environments.remove(&env_name);
            }
            let _ = save_collection_config(&collection_config, collection_path);
        }
    }

    Ok(environments.into_values().collect())
}

pub fn load_environments(collection_path: PathBuf) -> Result<Vec<Environment>, RelynxError> {
    let env_path = collection_path.join(HTTP_ENV_FILENAME);
    let private_env_path = collection_path.join(PRIVATE_HTTP_ENV_FILENAME);
    load_environments_from_files(
        Some(&env_path),
        Some(&private_env_path),
        Some(&collection_path),
    )
}

impl TryFrom<&EnvironmentVariable> for SingleEnvVarDescription {
    type Error = ();
    fn try_from(value: &EnvironmentVariable) -> Result<Self, Self::Error> {
        if value.description.is_none() {
            return Err(());
        }

        let description = value.description.clone().unwrap();
        if description.is_empty() {
            return Err(());
        }

        Ok(SingleEnvVarDescription {
            description,
            env_var_name: value.name.clone(),
            is_secret: false,
        })
    }
}

impl TryFrom<&EnvironmentSecret> for SingleEnvVarDescription {
    type Error = ();
    fn try_from(value: &EnvironmentSecret) -> Result<Self, Self::Error> {
        if value.description.is_none() {
            return Err(());
        }

        let description = value.description.clone().unwrap();

        Ok(SingleEnvVarDescription {
            description,
            env_var_name: value.name.clone(),
            is_secret: true,
        })
    }
}

pub fn save_environments(
    collection_path: PathBuf,
    environments: Vec<Environment>,
) -> Result<(), RelynxError> {
    let env_path = collection_path.join(HTTP_ENV_FILENAME);
    let private_env_path = collection_path.join(PRIVATE_HTTP_ENV_FILENAME);

    let mut env_file_structure = EnvFileStructure::new();
    let mut private_env_file_structure = EnvFileStructure::new();

    let config_file_path = Collection::config_file_path(&collection_path);

    let mut collection_config = load_collection_config(&config_file_path).unwrap_or_default();

    collection_config.env_var_descriptions = EnvVarDescriptions::new();

    for environment in environments {
        let env_key_values = env_file_structure
            .entry(environment.name.clone())
            .or_default();
        let private_env_key_values = private_env_file_structure
            .entry(environment.name.clone())
            .or_default();
        for variable in environment.variables.iter() {
            env_key_values.insert(variable.name.clone(), variable.initial_value.clone());
        }

        for secret in environment
            .secrets
            .iter()
            .filter(|secret| secret.persist_to_file)
        {
            private_env_key_values.insert(secret.name.clone(), secret.initial_value.clone());
        }

        // update descriptions
        let mut var_descriptions: Vec<SingleEnvVarDescription> = environment
            .variables
            .iter()
            .map(TryInto::<SingleEnvVarDescription>::try_into)
            .filter_map(Result::ok)
            .collect();

        var_descriptions.extend(
            environment
                .secrets
                .iter()
                .map(TryInto::<SingleEnvVarDescription>::try_into)
                .filter_map(Result::ok),
        );

        collection_config
            .env_var_descriptions
            .insert(environment.name, var_descriptions);
    }

    let env_file_content =
        serde_json::to_string::<EnvFileStructure>(&env_file_structure).map_err(|err| {
            log::error!("Could not serialize env_file_structure to string");
            log::error!("Public env file structure: {:?}", env_file_structure);
            log::error!("Io Error: {:?}", err);
            RelynxError::SaveEnvironmentsError
        })?;

    let private_env_file_content =
        serde_json::to_string::<EnvFileStructure>(&private_env_file_structure).map_err(|_| {
            // we do not log specifics as there are secrets within the file
            log::error!("Error saving private env file structure to file!");
            RelynxError::SaveEnvironmentsError
        })?;

    std::fs::write(env_path, env_file_content).map_err(|err| {
        log::error!("Could not write environment content to file");
        log::error!("Io Error: {:?}", err);

        RelynxError::SaveEnvironmentsError
    })?;

    std::fs::write(private_env_path, private_env_file_content).map_err(|_| {
        // we do not log specifics as there are secrets within the file
        log::error!("Error writing private environment to file");
        RelynxError::SaveEnvironmentsError
    })?;

    let result = save_collection_config(&collection_config, &config_file_path);
    if result.is_err() {
        log::error!("Could not save collection config after saving environment!");
        log::error!("Error: {:?}", result.unwrap_err());
        log::error!("Config: {:?}", collection_config);
        log::error!("Config path: {:?}", config_file_path);
    }

    Ok(())
}

fn load_env_structure(env_path: &PathBuf) -> Result<EnvFileStructure, RelynxError> {
    // the file does not need to exist, this is not an error
    if !env_path.exists() {
        return Ok(EnvFileStructure::new());
    }
    let env_file_content = std::fs::read_to_string(env_path);
    if env_file_content.is_err() {
        log::error!(
            "Load env structure, could not read environment file: '{}'",
            env_path.display()
        );
        return Err(RelynxError::LoadEnvironmentError);
    }
    let env_file_content = env_file_content.unwrap();
    let env_structure = serde_json::from_str::<EnvFileStructure>(&env_file_content).map_err(|err| {
        log::error!("Error in load_env_structure deserialize, could not load environment file at path: '{}', it seems to be malformed", env_path.display());
        log::error!("Serde Error: {:?}", err);
        RelynxError::LoadEnvironmentError
    })?;

    Ok(env_structure)
}
