use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::save_workspace;
use crate::error::RelynxError;
use crate::model::{
    Collection, ImportCollectionResult, ImportWarning, MessageSeverity, Multipart, Replaced,
    RequestBody, RequestModel, Workspace,
};
use crate::sanitize::sanitize_filename;
use crate::tree::{GroupOptions, RequestTreeNode};
use http_rest_file::model::{
    DataSource, DispositionField, HttpMethod, HttpVersion, RequestSettings, UrlEncodedParam,
    WithDefault,
};
use http_rest_file::Serializer;
use postman_collection::v2_1_0::*;
use postman_collection::PostmanCollection;

pub fn import(
    mut workspace: Workspace,
    import_path: PathBuf,
    result_path: PathBuf,
) -> Result<ImportCollectionResult, RelynxError> {
    match postman_collection::from_path(import_path) {
        Ok(collection) => {
            match collection {
                PostmanCollection::V1_0_0(_spec) => {
                    log::error!("Cannot import v1_0_0 collection!");
                    Err(RelynxError::TriedPostmanImportV1_0_0)
                }
                PostmanCollection::V2_0_0(_spec) => {
                    log::error!("Cannot import v1_0_0 collection!");
                    Err(RelynxError::TriedPostmanImportV2_0_0)
                }
                PostmanCollection::V2_1_0(spec) => {
                    let collection = postman_to_request_tree(result_path, spec);
                    // @TODO: handle not being able to save requests on file system
                    workspace.collections.push(collection.clone());
                    // @TODO: also save collection config there as well to mark it is actually a
                    // collection?
                    save_workspace(&workspace)?;

                    Ok(ImportCollectionResult { collection })
                }
            }
        }
        Err(err) => {
            log::error!("Could not import Postman collection, invalid format!");
            log::error!("Err: {:?}", err);
            Err(RelynxError::InvalidPostmanCollection)
        }
    }
}

#[allow(clippy::unused_unit)]
fn into_request_tree_node(
    item: &postman_collection::v2_1_0::Items,
    filename: &str,
    parent_path: &Path,
    import_warnings: &mut Vec<ImportWarning>,
) -> Result<RequestTreeNode, ()> {
    let path = parent_path.join(filename);

    if let Some(ref request) = item.request {
        let request_node = RequestTreeNode::new_request_node(
            transform_request(request, filename, &path, import_warnings),
            path.clone(),
        );

        let file_model = (&request_node).try_into().map_err(|_err| {
            // @TODO: error handling
            import_warnings.push(ImportWarning {
                rest_file_path: path.to_string_lossy().to_string(),
                is_group: false,
                severity: Some(MessageSeverity::Error),
                message: None,
            });
            ()
        })?;

        Serializer::serialize_to_file(&file_model).map_err(|_err| {
            // @TODO: error handling
            import_warnings.push(ImportWarning {
                rest_file_path: path.to_string_lossy().to_string(),
                is_group: false,
                severity: Some(MessageSeverity::Error),
                message: None,
            });

            ()
        })?;
        return Ok(request_node);
    }

    let mut group = RequestTreeNode::new_group(GroupOptions::FullPath(path.clone()));

    if !path.exists() {
        std::fs::create_dir(&path).map_err(|_err| {
            // @TODO: error handling
            import_warnings.push(ImportWarning {
                rest_file_path: path.to_string_lossy().to_string(),
                is_group: true,
                severity: Some(MessageSeverity::Error),
                message: None,
            });
            ()
        })?;
    }

    if let Some(ref children) = item.item {
        let item_names = item_names(children);
        group.children = children
            .iter()
            .enumerate()
            .map(|(index, child)| {
                into_request_tree_node(child, &item_names[index], &path, import_warnings)
            })
            .filter_map(|el| el.ok())
            .collect::<Vec<RequestTreeNode>>();
    }
    Ok(group)
}

fn next_free_name(
    base_name: &str,
    index: u32,
    existing_names: &HashMap<String, ()>,
    is_request: bool,
) -> String {
    let mut index = index;
    let extension = if is_request {
        http_rest_file::model::HttpRestFileExtension::Http.get_extension()
    } else {
        String::new()
    };

    loop {
        let name = format!("{}{}{}", base_name, index, extension);
        if existing_names.get(&name).is_none() {
            return name;
        }
        index += 1;
    }
}

fn item_names(items: &[Items]) -> Vec<String> {
    let mut names: HashMap<String, ()> = HashMap::new();
    for item in items.iter() {
        let mut name = sanitize_filename(item.name.clone().unwrap_or(String::new()));
        if item.request.is_some() {
            name = name + &http_rest_file::model::HttpRestFileExtension::Http.get_extension();
        }
        if name.is_empty() {
            if item.request.is_some() {
                name = next_free_name("Request_", 1, &names, true);
            }

            if item.item.is_some() {
                name = next_free_name("Group_", 1, &names, false);
            }
        }
        names.insert(name, ());
    }

    names.keys().cloned().collect()
}

fn postman_to_request_tree(
    import_result_path: PathBuf,
    collection: postman_collection::v2_1_0::Spec,
) -> Collection {
    let children_names = item_names(&collection.item);

    let mut import_warnings: Vec<ImportWarning> = Vec::new();

    collection
        .item
        .iter()
        .enumerate()
        .for_each(|(index, item)| {
            let _ = into_request_tree_node(
                item,
                &children_names[index],
                &import_result_path,
                &mut import_warnings,
            );
        });

    Collection {
        name: collection.info.name,
        path: import_result_path,
        description: match collection.info.description {
            Some(postman_collection::v2_1_0::DescriptionUnion::String(string)) => string,
            Some(postman_collection::v2_1_0::DescriptionUnion::Description(description)) => {
                description.content.unwrap_or_default()
            }
            None => String::new(),
        },
        path_exists: true,
        import_warnings,
        current_env_name: String::new(),
    }
}

fn transform_request(
    postman_request: &postman_collection::v2_1_0::RequestUnion,
    name: &str,
    request_path: &Path,
    import_warnings: &mut Vec<ImportWarning>,
) -> RequestModel {
    match postman_request {
        postman_collection::v2_1_0::RequestUnion::String(url) => RequestModel {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            rest_file_path: request_path.to_owned(),
            url: url.clone(),
            ..Default::default()
        },
        postman_collection::v2_1_0::RequestUnion::RequestClass(ref request_class) => {
            let method = match request_class.method {
                Some(ref string) => HttpMethod::new(string).into(),
                None => WithDefault::<HttpMethod>::default(),
            };

            let url = match request_class.url {
                Some(postman_collection::v2_1_0::Url::String(ref string)) => string.clone(),
                Some(postman_collection::v2_1_0::Url::UrlClass(ref url_class)) => {
                    url_class.raw.clone().unwrap_or(String::new())
                }
                None => String::new(),
            };

            // @TODO can http version be imported?
            let http_version = WithDefault::<HttpVersion>::default();

            let mut headers = match request_class.header {
                Some(HeaderUnion::String(ref string)) => {
                    // @TODO are these multiple headers or a single one? Schema doesn't give more
                    // info
                    let mut split = string.split(':');
                    let key = split.next().unwrap_or_default();
                    let value = split.next().unwrap_or_default();
                    vec![crate::model::Header {
                        key: key.to_string(),
                        value: value.to_string(),
                        active: true,
                    }]
                }
                Some(HeaderUnion::HeaderArray(ref array)) => {
                    // @TODO, description and enabled are ignored, maybe warn user that these are
                    // not imported
                    array
                        .iter()
                        .map(|header| crate::model::Header {
                            key: header.key.clone(),
                            value: header.value.clone(),
                            active: !header.disabled.unwrap_or(false),
                        })
                        .collect::<Vec<crate::model::Header>>()
                }
                None => {
                    vec![]
                }
            };

            let body: RequestBody =
                request_class
                    .body
                    .as_ref()
                    .map_or(RequestBody::None, |postman_body| {
                        match postman_body.mode {
                            Some(Mode::File) => {
                                let data = match postman_body.file {
                                    Some(File {
                                        content: Some(ref string),
                                        ..
                                    }) => DataSource::Raw(string.clone()),
                                    Some(File {
                                        src: Some(ref src_path),
                                        ..
                                    }) => DataSource::FromFilepath(src_path.clone()),
                                    // @TODO: maybe sho warning in this case?
                                    _ => DataSource::Raw(String::new()),
                                };
                                RequestBody::Raw { data }
                            },
                            Some(Mode::Raw) => RequestBody::Raw {
                                data: http_rest_file::model::DataSource::Raw(
                                    postman_body.raw.clone().unwrap_or_default(),
                                ),
                            },
                            Some(Mode::Formdata) => {
                                let mut parts: Vec<crate::model::Multipart> = Vec::new();

                                if let Some(formdata) = postman_body.formdata.clone() {
                                    formdata.iter().for_each(|form_param| {
                                        let mut headers = vec![];
                                        if let Some(content_type) = form_param.content_type.clone()
                                        {
                                            headers.push(crate::model::Header::new(
                                                "Content-Type",
                                                content_type,
                                            ));
                                        }
                                        let name = form_param.key.clone();
                                        // @TODO, @DECISION, disabled and description is ignored

                                        let form_parameter_type = form_param
                                            .form_parameter_type
                                            .clone()
                                            .unwrap_or_default();

                                        if let "file" = &form_parameter_type[..] {
                                            let file_src = match form_param.src.clone() {
                                                Some(FormParameterSrcUnion::File(file_src)) => {
                                                    file_src
                                                }
                                                Some(FormParameterSrcUnion::Files(files)) => {
                                                    import_warnings.push(ImportWarning {
                                                        rest_file_path: request_path
                                                            .to_string_lossy()
                                                            .to_string(),
                                                    severity: Some(MessageSeverity::Warn),
                                                    message: Some("Multiple files are present within the form parameters of the request body but only a single file can be imported".to_string()),

                                                        is_group: false,
                                                    });
                                                    files
                                                        .first()
                                                        .map(String::to_string)
                                                        .unwrap_or(String::new())
                                                }
                                                None => String::new(),
                                            };
                                        let filename = PathBuf::from(&file_src)
                                                .file_name()
                                                .unwrap_or_default()
                                                .to_string_lossy()
                                                .to_string();

                                        let disposition: http_rest_file::model::DispositionField = http_rest_file::model::DispositionField::new_with_filename(name,Some(filename));

                                        parts.push(crate::model::Multipart {
                                                headers,
                                            disposition,                                                                                                 data: DataSource::FromFilepath(
                                                    file_src
                                                ),
                                            })
                                        } else {
                                            parts.push(Multipart {
                                                headers,
                                                disposition: DispositionField::new(name),
                                                data: DataSource::Raw(
                                                    form_param.value.clone().unwrap_or_default(),
                                                ),
                                            })
                                        }
                                    });
                                }

                                RequestBody::Multipart {
                                    boundary: "----boundary----".to_string(),
                                    parts,
                                }
                            },
                            Some(Mode::Urlencoded) => {
                                let url_encoded_params: Vec<
                                    http_rest_file::model::UrlEncodedParam,
                                > = postman_body
                                    .urlencoded
                                    .clone()
                                    .unwrap_or_default()
                                    .iter()
                                    .map(|p| {
                                        // @TODO: p.description and p.disabled are ignored
                                        UrlEncodedParam {
                                            key: p.key.clone(),
                                            value: p.value.clone().unwrap_or(String::new()),
                                        }
                                    })
                                    .collect();
                                RequestBody::UrlEncoded { url_encoded_params }
                            },
                            Some(Mode::Graphql) => {
                                // @TODO: we modify headers, here, maybe create a `Headers` object and
                                // expose some methods
                                headers.push(crate::model::Header::new(
                                    "Content-Type",
                                    "application/json",
                                ));
                                let graphql = serde_json::to_string(&postman_body.graphql.clone().unwrap_or_default());
                                let data_source = match graphql {
                                    Ok(string) => DataSource::Raw(string),
                                    Err(err) => {
                                    log::error!("Error serializing postman graphql to string, err: {:?}", err);
                                    import_warnings.push(ImportWarning { rest_file_path: request_path.to_string_lossy().to_string(),  is_group: false, message: Some("GraphQl Body could not be imported".to_string()), severity: Some(MessageSeverity::Warn) });
                                    DataSource::Raw(String::new())
                                    }
                                };
                                RequestBody::Raw {
                                    data: data_source
                                }
                            },
                            None => RequestBody::None
                        }
                    });

            let description = request_class
                .description
                .clone()
                .map(|descr| match descr {
                    DescriptionUnion::String(string) => string,
                    DescriptionUnion::Description(description) => {
                        description.content.unwrap_or(String::new())
                    }
                })
                .unwrap_or(String::new());

            let http_version: Replaced<HttpVersion> = http_version.into();

            // @TODO: query params :(
            RequestModel {
                id: uuid::Uuid::new_v4().to_string(),
                name: name.to_string(),
                description,
                url,
                rest_file_path: request_path.to_owned(),
                body,
                method: method.unwrap_or_default(),
                headers,
                settings: RequestSettings::default(),
                query_params: vec![],
                http_version, // scripts are not imported from postman
                pre_request_script: None,
                response_handler: None,
                save_response: None,
            }
        }
    }
}
