use crate::{
    client::{client_model::Call, options::ClientOptions, Client},
    error::RelynxError,
    import::load_requests_from_file,
    model::Environment,
};

#[derive(Debug, PartialEq)]
pub struct RequestRun {
    calls: Vec<Call>,
}

#[allow(dead_code)]
pub fn load_and_run(
    request_path: &std::path::Path,
    options: &ClientOptions,
    environment: &Environment,
) -> Result<Vec<RequestRun>, RelynxError> {
    let request_models = load_requests_from_file(request_path).map_err(|err_details| {
        log::error!("Could not parse files, err_deails: {:?}", err_details);
        log::error!("Request path: '{}'", request_path.display());
        RelynxError::ParseErrorGeneric
    })?;

    let mut request_runs: Vec<RequestRun> = Vec::new();

    for request_model in request_models {
        let mut client = Client::new(None);
        let calls = client
            .execute(&request_model, options, Some(environment))
            .map_err(|http_err| {
                log::error!("Error during execute in load_and_run");
                log::error!("Http Error: {:?}", http_err);
                RelynxError::RequestSendError
            })?;
        request_runs.push(RequestRun { calls });
    }
    Ok(request_runs)
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use httptest::{matchers::*, responders::*, Expectation, Server};

    use crate::{
        client::options::ClientOptions,
        model::{Environment, EnvironmentVariable},
    };

    pub fn setup() -> (Server, Environment) {
        let server = Server::run();

        // The server provides server.addr() that returns the address of the
        // locally running server, or more conveniently provides a server.url() method
        // that gives a fully formed http url to the provided path.
        let mut server_url = server.url("").to_string();
        if server_url.ends_with("/") {
            server_url.pop();
        }
        let environment = Environment {
            name: "default".to_string(),
            variables: vec![EnvironmentVariable::new("base_url", server_url)],
            secrets: vec![],
        };

        // so we can define the path relative to this file, this helps as neovim has autocomplete
        // for paths when specifying request files
        let _ = std::env::set_current_dir("./src");

        (server, environment)
    }

    use super::load_and_run;
    #[test]
    pub fn test_simple_get() {
        let (server, environment) = setup();
        // Configure the server to expect a single GET /foo request and respond
        // with a 200 status code.
        server.expect(
            Expectation::matching(request::method_path("GET", "/devices"))
                .respond_with(status_code(200)),
        );

        let options = ClientOptions::default();

        let filepath = PathBuf::from("../tests/relynx-collection/simple/get_request.http");

        let result = load_and_run(&filepath, &options, &environment);

        assert!(result.is_ok());
        let runs = result.unwrap();

        assert_eq!(runs[0].calls[0].response.status, 200);
    }

    #[test]
    pub fn test_simple_get_with_params() {
        let (server, mut environment) = setup();
        // Configure the server to expect a single GET /foo request and respond
        // with a 200 status code.
        let matchers = all_of![
            request::method_path("GET", "/devices"),
            request::query("firstName=firstValue&secondName=secondValue&fromEnv=thirdValue")
        ];
        server.expect(Expectation::matching(matchers).respond_with(status_code(200)));

        environment
            .variables
            .push(EnvironmentVariable::new("fromEnv", "thirdValue"));

        let options = ClientOptions::default();

        let filepath =
            PathBuf::from("../tests/relynx-collection/simple/get_request_with_params.http");

        let result = load_and_run(&filepath, &options, &environment);

        assert!(result.is_ok());
        let runs = result.unwrap();

        assert_eq!(runs[0].calls[0].response.status, 200);
    }

    #[test]
    pub fn test_simple_get_with_headers() {
        let (server, mut environment) = setup();
        // Configure the server to expect a single GET /foo request and respond
        // with a 200 status code.
        let matchers = all_of![
            request::method_path("GET", "/get"),
            request::headers(contains(("user-agent", "relynx"))),
            request::headers(contains(("accept", "*/*"))),
            request::query("firstName=firstValue&secondName=secondValue"),
        ];
        server.expect(Expectation::matching(matchers).respond_with(status_code(200)));

        environment
            .variables
            .push(EnvironmentVariable::new("fromEnv", "thirdValue"));

        let options = ClientOptions::default();

        let filepath =
            PathBuf::from("../tests/relynx-collection/simple/get_request_with_headers.http");

        let result = load_and_run(&filepath, &options, &environment);

        assert!(result.is_ok());
        let runs = result.unwrap();

        assert_eq!(runs[0].calls[0].response.status, 200);
    }

    #[test]
    pub fn test_simple_post() {
        let (server, environment) = setup();
        // Configure the server to expect a single GET /foo request and respond
        // with a 200 status code.
        let matchers = all_of![request::method_path("POST", "/post"),];
        server.expect(Expectation::matching(matchers).respond_with(status_code(200)));

        let options = ClientOptions::default();

        let filepath = PathBuf::from("../tests/relynx-collection/simple/post_request.http");

        let result = load_and_run(&filepath, &options, &environment);

        assert!(result.is_ok());
        let runs = result.unwrap();

        assert_eq!(runs[0].calls[0].response.status, 200);
    }

    #[test]
    pub fn test_simple_custom_verb() {
        let (server, environment) = setup();
        // Configure the server to expect a single GET /foo request and respond
        // with a 200 status code.
        let matchers = all_of![request::method_path("CUSTOM_VERB", "/"),];
        server.expect(Expectation::matching(matchers).respond_with(status_code(200)));

        let options = ClientOptions::default();

        let filepath = PathBuf::from("../tests/relynx-collection/simple/custom_verb.http");

        let result = load_and_run(&filepath, &options, &environment);

        assert!(result.is_ok());
        let runs = result.unwrap();

        assert_eq!(runs[0].calls[0].response.status, 200);
    }

    #[test]
    pub fn test_post_form_url_encoded() {
        let (server, environment) = setup();
        // Configure the server to expect a single GET /foo request and respond
        // with a 200 status code.
        let matchers = all_of![
            request::method_path("POST", "/post"),
            request::headers(contains((
                "content-type",
                "application/x-www-form-urlencoded"
            ))),
            request::body("abc=def&ghi=jkl")
        ];
        server.expect(Expectation::matching(matchers).respond_with(status_code(200)));

        let options = ClientOptions::default();

        let filepath = PathBuf::from("../tests/relynx-collection/simple/post_form_urlencoded.http");

        let result = load_and_run(&filepath, &options, &environment);

        assert!(result.is_ok());
        let runs = result.unwrap();

        assert_eq!(runs[0].calls[0].response.status, 200);
    }

    #[test]
    pub fn test_post_json() {
        let (server, environment) = setup();
        // Configure the server to expect a single GET /foo request and respond
        // with a 200 status code.
        let matchers = all_of![
            request::method_path("POST", "/post"),
            request::headers(contains(("content-type", "application/json"))),
            request::body(json_decoded(eq(serde_json::json!({
                "key1": "value1",
                "sub1": {
                    "key2": "value2"
                }
            }))))
        ];
        server.expect(Expectation::matching(matchers).respond_with(status_code(200)));

        let options = ClientOptions::default();

        let filepath = PathBuf::from("../tests/relynx-collection/simple/post_json.http");

        let result = load_and_run(&filepath, &options, &environment);

        assert!(result.is_ok());
        let runs = result.unwrap();

        assert_eq!(runs[0].calls[0].response.status, 200);
    }

    #[test]
    pub fn test_post_multipart() {
        let (server, environment) = setup();
        // Configure the server to expect a single GET /foo request and respond
        // with a 200 status code.
        let matchers = all_of![
            request::method_path("POST", "/post"),
            request::headers(contains(("content-type", "application/json"))),
            request::body(json_decoded(eq(serde_json::json!({
                "key1": "value1",
                "sub1": {
                    "key2": "value2"
                }
            }))))
        ];
        server.expect(Expectation::matching(matchers).respond_with(status_code(200)));

        let options = ClientOptions::default();

        let filepath = PathBuf::from("../tests/relynx-collection/simple/post_json.http");

        let result = load_and_run(&filepath, &options, &environment);

        assert!(result.is_ok());
        let runs = result.unwrap();

        assert_eq!(runs[0].calls[0].response.status, 200);
    }
}
