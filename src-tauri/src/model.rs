use rspc::Type;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct Workspace {
    collections: Vec<Collection>,
}

impl Default for Workspace {
    fn default() -> Self {
        Workspace {
            collections: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct Collection {
    name: String,
    path: String,
    current_env_name: String,
    description: String,
    import_warnings: Vec<ImportWarning>,
}

// @TODO
#[derive(Serialize, Deserialize, Type, Debug)]
pub struct ImportWarning {}
