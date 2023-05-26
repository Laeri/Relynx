use rspc::Type;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{
    error::FrontendError,
    model::{Collection, RequestFileModel},
};
use http_rest_file::parser::Parser as RestFileParser;

pub mod postman;

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct LoadRequestsResult {
    pub file_models: Vec<RequestFileModel>,
    pub errs: Vec<FrontendError>,
}

pub fn load_requests_for_collection(
    collection: &Collection,
) -> Result<LoadRequestsResult, FrontendError> {
    let mut parse_errs: Vec<FrontendError> = Vec::new();
    let mut files: Vec<RequestFileModel> = Vec::new();
    for entry in WalkDir::new(&collection.path) {
        if let Ok(entry) = entry {
            if entry.file_type().is_file()
                && RestFileParser::has_valid_extension(
                    &entry.file_name().to_string_lossy().to_string(),
                )
            {
                match RestFileParser::parse_file(entry.path()) {
                    Ok(model) => files.push(model.into()),
                    Err(err) => parse_errs.push(err.into()),
                }
            }
        }
    }
    Ok(LoadRequestsResult {
        file_models: files,
        errs: parse_errs,
    })
}
