import { v4 as uuidv4 } from 'uuid';

import { RequestModel, RequestBody, HttpMethod, Environment } from '../bindings';
import { DEFAULT_HTTP_VERSION, newQueryParam, newRequestHeader, newRequestSettings, newEnvironment } from './model';


export const HTTP_METHODS = {
  GET: "GET" as HttpMethod,
  POST: "POST" as HttpMethod,
  PUT: "PUT" as HttpMethod,
  PATCH: "PATCH" as HttpMethod,
  DELETE: "DELETE" as HttpMethod,
  OPTIONS: "OPTIONS" as HttpMethod,
  HEAD: "HEAD" as HttpMethod,
  TRACE: "TRACE" as HttpMethod
}

export function requestMethodToString(method: HttpMethod) {
  if (isCustomMethod(method)) {
    return (method as {CUSTOM: string}).CUSTOM;
  } else {
    return method as string;
  }
}

export function isCustomMethod(method: HttpMethod) {
  if (typeof method === 'string' || method instanceof String) {
    return false;
  } else {
    return true;
  }
}

export type BodyType = "NONE" | "PLAIN_TEXT" | "FORM_URL_ENCODED" | "JSON" | "GRAPHQL" | "BINARY_FILE" | "FORM_DATA"

export type UUID = string

export const BodyTypes = {
  NONE: "NONE" as BodyType,
  PLAIN_TEXT: "PLAIN_TEXT" as BodyType,
  FORM_URL_ENCODED: "FORM_URL_ENCODED" as BodyType,
  JSON: "JSON" as BodyType,
  BINARY_FILE: "BINARY_FILE" as BodyType,
  FORM_DATA: "FORM_DATA" as BodyType,
  GRAPHQL: "GRAPHQL" as BodyType
}

export function translatedBodyType(bodyType: BodyType): string {
  switch (bodyType) {
    case BodyTypes.NONE:
      return 'None'
    case BodyTypes.PLAIN_TEXT:
      return 'Plain Text'
    case BodyTypes.FORM_URL_ENCODED:
      return "Form URL Encoded"
    case BodyTypes.JSON:
      return "JSON"
    case BodyTypes.BINARY_FILE:
      return "Binary File"
    case BodyTypes.FORM_DATA:
      return "Form Data"
    case BodyTypes.GRAPHQL:
      return "GraphQL"
    default:
      throw new Error("Missing translation in Request.translatedBodyTypes")
  }
}

// @TODO:
export function newRequestBody(): RequestBody {
  return "None";
}

export type FormProperty = {
  key: string,
  value: string,
  active: boolean
}

export function newFormProperty(): FormProperty {
  return {
    key: "",
    value: "",
    active: true
  }
}

export function newUUID(): UUID {
  return uuidv4() as UUID
}

export function newRequestModel(partial: Partial<RequestModel>): RequestModel {
  let request: RequestModel = {
    id: newUUID(),
    name: "",
    description: "",
    method: "GET",
    url: "",
    headers: [newRequestHeader(undefined)],
    query_params: [newQueryParam(undefined)],
    body: newRequestBody(),
    rest_file_path: "",
    http_version: { value: DEFAULT_HTTP_VERSION, is_replaced: true },
    settings: newRequestSettings(undefined)
  }

  if (partial) {
    request = { ...request, ...partial };
  }
  return request;
}

export function updatedRequestModel(requestModel: RequestModel, partial: Partial<RequestModel>): RequestModel {
  let request = newRequestModel(requestModel);
  Object.assign(request, partial);
  return request
}

export function getUpdatedEnvironment(old: Environment, partial: Partial<Environment>): Environment {
  let environment = newEnvironment(old);
  Object.assign(environment, partial);
  return environment;
}

const HttpStatusText: { [key: string]: string; } = {
  '200': 'OK',
  '201': 'Created',
  '202': 'Accepted',
  '203': 'Non-Authoritative Information',
  '204': 'No Content',
  '205': 'Reset Content',
  '206': 'Partial Content',
  '300': 'Multiple Choices',
  '301': 'Moved Permanently',
  '302': 'Found',
  '303': 'See Other',
  '304': 'Not Modified',
  '305': 'Use Proxy',
  '306': 'Unused',
  '307': 'Temporary Redirect',
  '400': 'Bad Request',
  '401': 'Unauthorized',
  '402': 'Payment Required',
  '403': 'Forbidden',
  '404': 'Not Found',
  '405': 'Method Not Allowed',
  '406': 'Not Acceptable',
  '407': 'Proxy Authentication Required',
  '408': 'Request Timeout',
  '409': 'Conflict',
  '410': 'Gone',
  '411': 'Length Required',
  '412': 'Precondition Required',
  '413': 'Request Entry Too Large',
  '414': 'Request-URI Too Long',
  '415': 'Unsupported Media Type',
  '416': 'Requested Range Not Satisfiable',
  '417': 'Expectation Failed',
  '418': 'I\'m a teapot',
  '429': 'Too Many Requests',
  '500': 'Internal Server Error',
  '501': 'Not Implemented',
  '502': 'Bad Gateway',
  '503': 'Service Unavailable',
  '504': 'Gateway Timeout',
  '505': 'HTTP Version Not Supported',
};

export function getStatusTextForCode(codeStr: string) {
  let text = HttpStatusText[codeStr]
  return text ?? ""
}


