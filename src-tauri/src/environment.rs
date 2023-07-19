use std::{collections::HashMap, path::PathBuf};

use crate::{
    config::{load_collection_config, save_collection_config},
    error::{DisplayErrorKind, FrontendError},
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
) -> Result<Vec<Environment>, FrontendError> {
    let mut environments: HashMap<String, Environment> = HashMap::new();

    if let Some(env_file_path) = env_file_path {
        let env_structure = load_env_structure(env_file_path)?;

        for (env_name, key_val_map) in env_structure.iter() {
            let variables: Vec<EnvironmentVariable> = key_val_map
                .into_iter()
                .map(|(var_name, value)| EnvironmentVariable {
                    name: var_name.to_string(),
                    initial_value: value.to_string(),
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
                    name: var_name.to_string(),
                    initial_value: value.to_string(),
                    current_value: None,
                    description: None, // @TODO: get description from collectionconfig :(
                    persist_to_file: true,
                })
                .collect();
            environment.secrets = secrets;
        }
    }

    if let Some(collection_path) = collection_path {
        let collection_config = load_collection_config(&collection_path);
        if collection_config.is_ok() {
            let mut collection_config = collection_config.unwrap();
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
            // @TODO: error handling
            let _ = save_collection_config(&collection_config, &collection_path);
        } else {
            // @TODO: log warning
        }
    }

    Ok(environments.into_values().into_iter().collect())
}

pub fn load_environments(collection_path: PathBuf) -> Result<Vec<Environment>, FrontendError> {
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
        if description == "" {
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
) -> Result<(), FrontendError> {
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
            env_key_values.insert(
                variable.name.clone(),
                // @TODO: log error
                variable.initial_value.clone(),
            );
        }

        for secret in environment
            .secrets
            .iter()
            .filter(|secret| secret.persist_to_file)
        {
            private_env_key_values.insert(
                secret.name.clone(),
                // @TODO: log error
                secret.initial_value.clone(),
            );
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

    // @TODO: log error, env_file_structure?
    let env_file_content =
        serde_json::to_string::<EnvFileStructure>(&env_file_structure).map_err(|_err| {
            let msg = "Could not save environments to file";
            FrontendError::new_with_message(DisplayErrorKind::SaveEnvironmentsError, msg)
        })?;

    let private_env_file_content =
        serde_json::to_string::<EnvFileStructure>(&private_env_file_structure).map_err(|_err| {
            let msg = "Could not save environments to file";
            FrontendError::new_with_message(DisplayErrorKind::SaveEnvironmentsError, msg)
        })?;

    // @TODO: log error
    std::fs::write(&env_path, env_file_content).map_err(|_err| {
        let msg = format!(
            "Could not save environment to file: '{}'",
            env_path.to_string_lossy()
        );
        FrontendError::new_with_message(DisplayErrorKind::SaveEnvironmentsError, msg)
    })?;

    std::fs::write(&private_env_path, private_env_file_content).map_err(|_err| {
        let msg = format!(
            "Could not save environment to file: '{}'",
            env_path.to_string_lossy()
        );
        FrontendError::new_with_message(DisplayErrorKind::SaveEnvironmentsError, msg)
    })?;

    // @TODO log error
    let _ = save_collection_config(&collection_config, &config_file_path);

    Ok(())
}

fn load_env_structure(env_path: &PathBuf) -> Result<EnvFileStructure, FrontendError> {
    // the file does not need to exist, this is not an error
    if !env_path.exists() {
        return Ok(EnvFileStructure::new());
    }
    let env_file_content = std::fs::read_to_string(env_path);
    if env_file_content.is_err() {
        let msg = format!(
            "Could not read environment file: '{}'",
            env_path.to_string_lossy(),
        );
        return Err(FrontendError::new_with_message(
            DisplayErrorKind::LoadEnvironmentsError,
            msg,
        ));
    }
    let env_file_content = env_file_content.unwrap();
    let env_structure =
        serde_json::from_str::<EnvFileStructure>(&env_file_content).map_err(|_err| {
            let msg = format!(
                "Could not load environment file: '{}', it seems to be malformed",
                env_path.to_string_lossy(),
            );
            FrontendError::new_with_message(DisplayErrorKind::LoadEnvironmentsError, msg)
        })?;

    Ok(env_structure)
}
