import { v4 as uuidv4 } from 'uuid';

import { RequestModel, RequestBody, HttpMethod, Environment, Header, DataSource, Multipart, UrlEncodedParam, QueryParam } from '../bindings';
import { QueryParams } from '../components/QueryParams';
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
    return (method as { CUSTOM: string }).CUSTOM;
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


export type UUID = string

export type RequestBodyNone = "None";
export type RequestBodyMultipart = { Multipart: { boundary: string; parts: Multipart[] } };
export type RequestBodyUrlEncoded = { UrlEncoded: { url_encoded_params: UrlEncodedParam[] } };
export type RequestBodyRaw = { Raw: { data: DataSource<string> } };

export type DataSourceRaw<T> = { Raw: T };
export type DataSourceFromFilepath = { FromFilepath: string };


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
    settings: newRequestSettings(undefined),
    save_response: null,
    pre_request_script: null,
    response_handler: null
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

export function get_content_type(requestModel: RequestModel): string | undefined {
  let contentTypeHeader = requestModel.headers.find((header: Header) => {
    return header.key.toLowerCase() == "content-type";
  });
  return contentTypeHeader?.value;
}

export function hasContentType(requestModel: RequestModel, contentType: string): boolean {
  let requestContentType = get_content_type(requestModel);
  if (requestContentType === undefined || requestContentType === null) {
    return false;
  }
  return requestContentType.split(';').some((part: string) => part === contentType);
}

export function getRawText(requestBody: RequestBody): string | undefined {
  let rawTextBody = requestBody as RequestBodyRaw;
  return (rawTextBody.Raw.data as DataSourceRaw<string>).Raw
}


export function isDataSourceFromFile(dataSource: DataSource<String>): boolean {
  let fileSource = dataSource as DataSourceFromFilepath;
  return fileSource.FromFilepath !== undefined;
}

export function newMultipartPart(): Multipart {
  return { data: { Raw: "" }, disposition: { name: "", filename: null, filename_star: null }, headers: [] };
}


export function removeHeader(request: RequestModel, headerKey: string) {
  let headerKeyLower = headerKey.toLowerCase();
  request.headers = request.headers.filter((header: Header) => header.key.toLowerCase() !== headerKeyLower);
}

// @TODO: handle multiple headers with same key...
export function setHeader(request: RequestModel, header: Header) {
  let headerKeyLower = header.key.toLowerCase();
  let index = request.headers.findIndex((header: Header) => header.key.toLowerCase() == headerKeyLower);
  if (index !== -1) {
    request.headers[index] = header;
  } else {
    request.headers.push(header);
  }
}

function searchParamsToQueryParams(urlParams: URLSearchParams): QueryParam[] {
  let result = Array.from(urlParams.entries()).map(([key, value]: [string, string]) => {
    let param: QueryParam = {
      key: key,
      value: value,
      active: true
    }
    return param;
  });
  return result;
}

function queryParamsToString(queryParams: QueryParam[]): string {
  return queryParams.map((queryParam: QueryParam) => {
    return queryParam.key + "=" + queryParam.value;
  }).join("&")
}

export function changeUrlParams(url: string, oldParam: QueryParam | undefined, newParam: QueryParam | undefined): string {
  let urlSplit = url.split('?');
  if (urlSplit.length == 1) {
    urlSplit = [url, ""];
  }

  let searchQuery = new URLSearchParams(urlSplit[1]);
  let queryParams = searchParamsToQueryParams(searchQuery);
  // change url param
  if (oldParam && newParam) {
    let index = queryParams.findIndex((param: QueryParam) => param.key === oldParam.key);
    if (index !== -1) {
      queryParams[index] = newParam;
    }

    // remove param
  } else if (oldParam && !newParam) {
    let index = queryParams.findIndex((param: QueryParam) => param.key === oldParam.key);
    if (index !== -1) {
      queryParams.splice(index, 1);
    }
    // remove url param
  } else if (!oldParam && newParam) {
    queryParams.push(newParam);
  }

  if (queryParams.length > 0) {
    let queryString = queryParamsToString(queryParams);
    return urlSplit[0] + "?" + queryString;
  } else {
    return urlSplit[0];
  }
}

export function changeRequestUrlParams(request: RequestModel, oldParam: QueryParam | undefined, newParam: QueryParam | undefined) {
  let url = changeUrlParams(request.url, oldParam, newParam);
  request.url = url;
  let queryParams = extractQueryParamsFromUrl(request);
  request.query_params = queryParams;
}

export function extractQueryParamsFromUrl(request: RequestModel): QueryParam[] {
  try {
    let url = new URL(request.url);
    return searchParamsToQueryParams(url.searchParams);
  } catch (err) {
    let split = request.url.split('?');
    if (split.length > 1) {
      let searchQuery = new URLSearchParams(split[1]);
      return searchParamsToQueryParams(searchQuery);
    } else {
      return []
    }
  }
}

