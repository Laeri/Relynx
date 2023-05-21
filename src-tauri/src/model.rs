use rspc::Type;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct Workspace {
    pub collections: Vec<Collection>,
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
    pub name: String,
    pub path: String,
    pub current_env_name: String,
    pub description: String,
    pub import_warnings: Vec<ImportWarning>,
}

// @TODO
#[derive(Serialize, Deserialize, Type, Debug)]
pub struct ImportWarning {}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct AddCollectionsResult {
    pub workspace: Workspace,
    pub any_collections_found: bool,
    pub num_imported: i32,
    pub errored_collections: Vec<String>, // @TODO
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct ImportCollectionResult {
    pub collection: Collection,
    // @TODO: environment
    // @TODO: requestTree
    // @TODO: collectionConfig
    pub import_warnings: Vec<ImportWarning>,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct CollectionConfig {
    pub name: String
}


