use crate::error::FrontendError;
use crate::model::{ImportCollectionResult, Workspace};
use postman_collection::v2_1_0::*;
use postman_collection::PostmanCollection;

use http_rest_file::model::{
    Comment, CommentKind, Header, HttpMethod, HttpVersion, Request, RequestBody, RequestLine,
    RequestSettings, RequestTarget, WithDefault,
};
pub fn import(
    workspace: Workspace,
    import_path: String,
    result_path: String,
) -> Result<ImportCollectionResult, FrontendError> {
    if let Some(path) = std::env::args().nth(1) {
        match postman_collection::from_path(path) {
            Ok(collection) => {
                match collection {
                    PostmanCollection::V1_0_0(spec) => {
                        println!("Found v1.0.0 collection with the name: {}", spec.name);
                    }
                    PostmanCollection::V2_0_0(spec) => {
                        println!("Found v2.0.0 collection with the name: {}", spec.info.name);
                    }
                    PostmanCollection::V2_1_0(spec) => {
                        println!("Found v2.1.0 collection with the name: {}", spec.info.name);
                    }
                }
                //println!("{}", postman_collection::to_json(&spec).unwrap());
            }
            Err(e) => {
                // @TODO error handling
                return Err(FrontendError::new(
                    crate::error::DisplayErrorKind::ImportPostmanError,
                ));
            }
        }
    }

    //Ok(ImportCollectionResult { collection: , import_warnings: () })
    todo!("TODO implement");
}

fn postman_to_relynx_collection(collection: postman_collection::v2_1_0::Spec) -> Vec<Request> {
    let mut requests: Vec<Request> = Vec::new();
    let mut items: Vec<Items> = collection.item.clone();
    while items.is_empty() {
        let item = items.remove(0);
        if let Some(request) = item.request {
            let relynx_request = transform_request(request);
            requests.push(relynx_request);
            continue;
        }
        if let Some(children) = item.item {
            items.extend(children);
        }
    }
    requests
}

fn group(item: Items) {}

fn transform_request(request: postman_collection::v2_1_0::RequestUnion) -> Request {
    match request {
        postman_collection::v2_1_0::RequestUnion::String(url) => {
            let mut request = Request::default();
            let mut request_line = RequestLine::default();
            // @TODO what about relative uri?
            request.request_line.target = RequestTarget::Absolute { uri: url };
            request
        }
        postman_collection::v2_1_0::RequestUnion::RequestClass(request_class) => {
            let method = match request_class.method {
                Some(string) => HttpMethod::new(&string).into(),
                None => WithDefault::<HttpMethod>::default(),
            };

            let url = match request_class.url {
                Some(postman_collection::v2_1_0::Url::String(string)) => string.to_string(),
                Some(postman_collection::v2_1_0::Url::UrlClass(url_class)) => {
                    url_class.raw.unwrap_or(String::new())
                }
                None => String::new(),
            }
            .to_string();

            let request_target =
                RequestTarget::parse(&url).unwrap_or(RequestTarget::InvalidTarget(url.clone()));

            // @TODO can http version be imported?
            let http_version = WithDefault::<HttpVersion>::default();
            let request_line = RequestLine {
                method,
                target: request_target,
                http_version,
            };

            let headers = match request_class.header {
                // @TODO is this one header or a list of them delimited by semicolong?
                Some(HeaderUnion::String(string)) => {
                    // @TODO check if one header or all
                    //string.split(';').collect::<String>().split('=')
                    vec![]
                }
                Some(HeaderUnion::HeaderArray(array)) => {
                    // @TODO, description and enabled are ignored, maybe warn user that these are
                    // not imported
                    array
                        .iter()
                        .map(|header| http_rest_file::model::Header {
                            key: header.key.clone(),
                            value: header.value.clone(),
                        })
                        .collect::<Vec<Header>>()
                }
                None => {
                    vec![]
                }
            };

            // @TODO todo!("body");
            let description = request_class
                .description
                .map(|descr| match descr {
                    DescriptionUnion::String(string) => string.to_string(),
                    DescriptionUnion::Description(description) => {
                        description.content.map_or(String::new(), |v| v.to_string())
                    }
                })
                .unwrap_or(String::new());

            Request {
                name: Some(url.clone()),
                comments: vec![Comment {
                    value: description,
                    kind: CommentKind::DoubleSlash,
                }],
                request_line,
                // scripts are not imported from postman
                pre_request_script: None,
                response_handler: None,
                settings: RequestSettings::default(), // @TODO which settings can we use?
                // @TODO maybe rename redirect in http_rest_file library to output_redirect
                redirect: None,
                headers,
                body: RequestBody::None, // @TODO
            }
        }
    }
}
