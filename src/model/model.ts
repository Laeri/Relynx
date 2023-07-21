import { Workspace, Collection, RequestModel, Header, HttpVersion, QueryParam, RequestResult, RequestSettings, Environment } from "../bindings";

// @TODO: check
export const DEFAULT_HTTP_VERSION: HttpVersion = { major: 1, minor: 1 };

export function newWorkspace(partial?: Partial<Workspace>): Workspace {
  if (!partial) {
    partial = {};
  }
  return { collections: [], ...partial };
}

export function newCollection(): Collection {
  return {
    name: "",
    path: "",
    description: "",
    import_warnings: [],
    current_env_name: ""
  };
}

export function updatedRequestModel(model: RequestModel, partial: Partial<RequestModel>): RequestModel {
  return {
    ...model,
    ...partial
  };
}

export function newQueryParam(partial: undefined | Partial<QueryParam>): QueryParam {
  let param = {
    key: "",
    value: "",
    active: true
  };
  if (partial) {
    param = { ...param, ...partial };
  }
  return param;
}

export function newRequestHeader(partial: undefined | Partial<Header>): Header {
  let header = {
    key: "",
    value: "",
    active: true
  };

  if (partial) {
    header = { ...header, ...partial }
  }
  return header;
}

// @TODO check defaults here
export function newRequestSettings(partial: undefined | Partial<RequestSettings>): RequestSettings {
  let requestSettings: RequestSettings = {
    no_log: false,
    no_redirect: false,
    no_cookie_jar: false,
  };
  if (partial) {
    requestSettings = { ...requestSettings, ...partial };
  }
  return requestSettings;
}

export function newEnvironment(partial: undefined | Partial<Environment>): Environment {
  // @TODO check nameclash!
  let environment: Environment = {
    name: "New Environment",
    secrets: [],
    variables: [],
  }

  if (partial) {
    environment = { ...environment, ...partial };
  }
  return environment;
}

export interface Cookie {
  value: string
}

export function getCookies(requestModel: RequestModel): Cookie[] {
  let cookies = requestModel.headers.filter((header: Header) => {
    return header.key.toLowerCase() == "cookie";
  }).flatMap((header: Header) => {
    return header.value.split(',');
  }).map((cookieValue: string) => {
    return { value: cookieValue };
  });

  return cookies;
}
