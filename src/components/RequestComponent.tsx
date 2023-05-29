import { useContext, useEffect, useMemo, useState } from "react";
import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";
import { Dropdown } from "primereact/dropdown";
import { KeyValueRow } from "./KeyValueRow";
// @TODO: import { RequestBodyComp } from "./RequestBodyComp";
import { InputTextarea } from "primereact/inputtextarea";
import { ProgressSpinner } from "primereact/progressspinner";
import { TabPanel, TabView } from "primereact/tabview";
import { useRequestModelStore } from "../stores/requestStore";
import { backend } from '../rpc';
import { RequestModel, QueryParam, Header, Collection, ImportWarning, RunRequestCommand, RequestResult, EnvironmentVariable, EnvironmentSecret, HttpMethod } from '../bindings';
//import {getAllRequestsFromTree, getRequestsInSameGroup} from "../common/treeUtils";
import { ToastContext } from "../App";
import { catchError } from "../common/errorhandling";
//import RequestResult = models.RequestResult;
import { ResultDisplay } from "./ResultDisplay";
import { scrollMainToTop } from "../common/common";
import { Message } from "primereact/message";
import { hasInvalidFileBody } from "../common/requestUtils";
import { WarningCollapsible } from "./WarningCollapsible";
import { updatedRequestModel, newQueryParam, newRequestHeader } from '../model/model';
import { getAllRequestsFromTree } from "../common/treeUtils";

interface ComponentProps {
}

export function RequestComponent(_props: ComponentProps) {

  const currentRequest = useRequestModelStore((state) => state.currentRequest as RequestModel);
  const storeUpdateRequestAndTree = useRequestModelStore((state) => state.storeUpdateRequestAndTree);
  /* const url = useRequestModelStore((state) => state.currentRequest?.Url as string); // we are sure that we have a request here
  const requestType = useRequestModelStore((state) => state.currentRequest?.RequestType as RequestType);
  const queryParams = useRequestModelStore((state) => state.currentRequest?.QueryParams as QueryParam[]);
  const requestHeaders = useRequestModelStore((state) => state.currentRequest?.RequestHeaders as RequestHeader[]); */

  const requestResult = useRequestModelStore((state) => state.requestResult);
  const workspace = useRequestModelStore((state) => state.workspace);
  const updateWorkspace = useRequestModelStore((state) => state.updateWorkspace);

  // @TODO const updateRequestResult = useRequestModelStore((state) => state.updateRequestResult);
  const clearResultText = useRequestModelStore((state) => state.clearRequestResult);

  const currentCollection = useRequestModelStore((state) => state.currentCollection);

  const currentEnvironment = useRequestModelStore((state) => state.currentEnvironment);

  const requestTree = useRequestModelStore((state) => state.requestTree);

  const [activeIndex, setActiveIndex] = useState<number>(0);

  const toast = useContext(ToastContext);

  const [isSendingRequest, setIsSendingRequest] = useState<boolean>(false);

  const [nameError, setNameError] = useState<string | undefined>(undefined);


  const [tmpRequestName, setTmpRequestName] = useState<string>('');

  const [importWarnings, setImportWarnings] = useState<ImportWarning[]>([]);


  useEffect(() => {
    scrollMainToTop();
    if (!currentRequest) {
      return
    }
    setTmpRequestName(currentRequest.name);

    let importWarnings: ImportWarning[] = (currentCollection as Collection).import_warnings.filter((import_warning: ImportWarning) => {
      return import_warning.request_name === currentRequest.name && import_warning.rest_file_path === currentRequest.rest_file_path
    });
    setImportWarnings(importWarnings);
  }, [currentRequest]);

  // send request data to backend and perform libcurl request
  function doRequest() {
    // TODO: validate model
    let backendRequest: RunRequestCommand = {
      request: currentRequest,
      environment: null  //@TODO: check
    }

    // filter out variables with duplicated key names (keep the first one)
    if (currentEnvironment) {
      let backendEnvironment = structuredClone(currentEnvironment);

      // @TODO: check
      let presentVarNames: { [name: string]: boolean } = {};
      backendEnvironment.variables = backendEnvironment.variables.filter((variable: EnvironmentVariable) => {
        if (presentVarNames[variable.name]) {
          return false;
        } else {
          presentVarNames[variable.name] = true;
          return true;
        }
      });

      backendEnvironment.secrets = backendEnvironment.secrets.filter((secret: EnvironmentSecret) => {
        if (presentVarNames[secret.name]) {
          return false;
        } else {
          presentVarNames[secret.name] = true;
          return true;
        }
      });

      backendRequest.environment = backendEnvironment;
    }

    // reset request result before the request
    // @TODO: updateRequestResult(newRequestResult());
    setIsSendingRequest(true);
    backend.runRequest(backendRequest).then((result: RequestResult) => {
      // @TODO: updateRequestResult(result);
    }).catch(catchError(toast)).finally(() => {
      setIsSendingRequest(false);
    });
  }

  function updateRequest(newRequest: RequestModel, valid: boolean = true) {
    console.log('update request', newRequest);
    if (!currentCollection || !currentRequest) {
      return
    }

    let allRequests: RequestModel[] = getAllRequestsFromTree(requestTree);

    // find all requests with the same path which means they are in the same file
    // @SPEED, we have to parse the tree everytime we do this
    let requestsInSameFile = allRequests.filter((request: RequestModel) => request.rest_file_path == newRequest.rest_file_path || request.id == newRequest.id);
    console.log('requestsInSameFile: ', requestsInSameFile);

    // replace the new request within our requests here
    requestsInSameFile = requestsInSameFile.map((request: RequestModel) => {
      if (request.id === newRequest.id) {
        return newRequest
      } else {
        return request
      }
    });

    console.log('requestsInSameFile2: ', requestsInSameFile);
    backend.saveRequest(requestsInSameFile, currentCollection, currentRequest.name).then(() => {
      storeUpdateRequestAndTree(newRequest)
    }).catch(catchError(toast));

  }

  function updateRequestName(name: string) {
    if (!currentRequest) {
      return
    }
    let newRequest = updatedRequestModel(currentRequest, { name: name });
    setTmpRequestName(name);
    let valid = validateRequestName(name);
    if (valid) {
      updateRequest(newRequest);
    }
  }

  function validateRequestName(name: string): boolean {
    if (name === '') {
      setNameError("The name cannot be empty");
      return false;
    }

    let requests: RequestModel[] = [] // @TODO getRequestsInSameGroup(currentRequest.Id, requestTree)
    if (requests.map((request: RequestModel) => request.name).includes(name)) {
      setNameError('There exists already a request with the same name in this group')
      return false;
    }
    setNameError(undefined);
    return true;
  }

  function updateUrl(url: string) {
    let newRequest = updatedRequestModel(currentRequest, { url: url });
    updateRequest(newRequest);
  }

  function updateDescription(description: string) {
    let newRequest = updatedRequestModel(currentRequest, { description: description });
    updateRequest(newRequest);
  }

  function updateRequestType(method: HttpMethod) {
    let newRequest = updatedRequestModel(currentRequest, { method: method });
    updateRequest(newRequest);
  }

  function updateQueryParam(oldParam: QueryParam, newParam: QueryParam) {
    let newQueryParams = [...currentRequest.query_params];
    let index = newQueryParams.indexOf(oldParam);
    newQueryParams[index] = newParam;
    let newRequest = updatedRequestModel(currentRequest, { query_params: newQueryParams });
    updateRequest(newRequest);
  }

  function updateQueryParamKey(queryParam: QueryParam, key: string) {
    let param: QueryParam = newQueryParam({ ...queryParam, key: key });
    updateQueryParam(queryParam, param);
  }

  function updateQueryParamValue(queryParam: QueryParam, value: string) {
    let param: QueryParam = newQueryParam({ ...queryParam, value: value });
    updateQueryParam(queryParam, param);
  }

  function updateQueryParamActive(queryParam: QueryParam, active: boolean) {
    let param = newQueryParam({ ...queryParam, active: active });
    updateQueryParam(queryParam, param);
  }

  function updateRequestHeader(oldHeader: Header, newHeader: Header) {
    let newRequestHeaders = [...currentRequest.headers];
    let index = newRequestHeaders.indexOf(oldHeader);
    newRequestHeaders[index] = newHeader;

    let newRequestModel = updatedRequestModel(currentRequest, { headers: newRequestHeaders });
    updateRequest(newRequestModel);
  }

  function addQueryParam() {
    let newRequestModel: RequestModel = updatedRequestModel(currentRequest, {
      query_params: [...currentRequest.query_params, newQueryParam(undefined)]
    });
    updateRequest(newRequestModel);
  }

  function removeQueryParam(queryParam: QueryParam) {
    let newQueryParams = currentRequest.query_params.filter((current: QueryParam) => current !== queryParam);
    let newRequestModel: RequestModel = updatedRequestModel(currentRequest, { query_params: newQueryParams });
    updateRequest(newRequestModel);
  }

  function addHeader() {
    let newRequestModel: RequestModel = updatedRequestModel(
      currentRequest, {
      headers: [...currentRequest.headers, newRequestHeader(undefined)]
    });
    updateRequest(newRequestModel);
  }

  function removeHeader(requestHeader: Header) {
    let newRequestHeaders = currentRequest.headers.filter((current: Header) => current !== requestHeader)
    let newRequestModel: RequestModel = updatedRequestModel(
      currentRequest, {
      headers: newRequestHeaders
    });
    updateRequest(newRequestModel);
  }

  function updateHeaderKey(requestHeader: Header, key: string) {
    let header = newRequestHeader({ ...requestHeader, key: key });
    updateRequestHeader(requestHeader, header);
  }

  function updateHeaderValue(requestHeader: Header, value: string) {
    let header = newRequestHeader({ ...requestHeader, value: value });
    updateRequestHeader(requestHeader, header);
  }

  function updateHeaderActive(requestHeader: Header, active: boolean) {
    let header = newRequestHeader({ ...requestHeader, active: active });
    updateRequestHeader(requestHeader, header);
  }

  const requestTypeOptions: { name: string, value: string }[] = []; /* @TODO Object.entries(requestTypes).map(([key, value]) => {
    return { name: key, value: value };
  }); */

  const copyResultToClipboard = () => {
    backend.copyToClipboard(requestResult?.result ?? '').then(() => {
      toast.showInfo("Copied to clipboard", "");
    }).catch((_err) => {
      toast.showError("Could not copy content to clipboard", "");
    });
  }

  // Result needs to be memoized because if it is updated frequently it will freeze inputs
  const resultDisplay = useMemo(() => {
    return <ResultDisplay requestResult={requestResult} copyResultToClipboard={copyResultToClipboard}
      clearResultText={clearResultText} />
  }, [requestResult]);

  const clearImportWarnings = () => {
    if (!currentCollection) {
      return
    }
    let newWorkspace = structuredClone(workspace);
    let newCollection = structuredClone(currentCollection);
    newCollection.import_warnings = newCollection.import_warnings.filter((importWarning: ImportWarning) => {
      return (importWarning.request_name !== currentRequest.name && importWarning.rest_file_path !== currentRequest.rest_file_path)
    });

    newWorkspace.collections = newWorkspace.collections.map((currentCol: Collection) => {
      if (currentCol.path === currentCollection.path) {
        return newCollection;
      } else {
        return currentCol;
      }
    });

    updateWorkspace(newWorkspace);
    backend.updateWorkspace(newWorkspace).then(() => {
      // do nothing
    }).catch(catchError(toast));
  }

  return (
    <>
      <div className={'fade-in-fast'} style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
        {/*TODO: Remove header or make it an inline edit?*/}
        {/*TODO: Also replace name and request url?*/}
        <h1 style={{ marginTop: '20px' }}>Request</h1>

        {/*Only display invalid file body if there are no import warnings as they might already contain this type of error*/}
        {importWarnings.length == 0 && hasInvalidFileBody(currentRequest) &&
          <Message style={{ marginTop: '10px' }}
            text={"You have a request body from a file that is missing. Set the file path of the request body file."}
            severity={"warn"}></Message>
        }
        <div style={{
          width: '100%',
          display: 'flex',
          justifyContent: 'flex-start',
          marginTop: '40px',
          marginBottom: '20px'
        }}>
          <div style={{ width: '100%', display: 'flex', flexDirection: 'column' }}>
            <InputText style={{ maxWidth: '500px' }} value={tmpRequestName}
              onChange={(e) => updateRequestName(e.target.value)}
              placeholder={"Name"} />
            {nameError !== '' &&
              <span className={"invalid mt-2"} style={{ textAlign: 'left' }}>{nameError}</span>

            }
          </div>
        </div>
        <div className="input" style={{ display: 'flex', marginTop: '30px' }}>
          <Dropdown optionLabel="name" value={currentRequest.method} options={requestTypeOptions}
            onChange={(e) => updateRequestType(e.value)} />
          <InputText value={currentRequest.url} onChange={(e) => updateUrl(e.target.value)} placeholder={"Url"}
            style={{ marginLeft: '20px', flexBasis: '35%' }} disabled={isSendingRequest} />
          <Button label="Send Request" onClick={doRequest} className="p-button-outlined"
            style={{ marginLeft: '20px' }} disabled={isSendingRequest} />
          {
            isSendingRequest && <ProgressSpinner style={{ maxHeight: '30px' }} />
          }
        </div>
        {
          importWarnings.length > 0 &&
          <WarningCollapsible collection={currentCollection as Collection} importWarnings={importWarnings}
            onClearWarnings={clearImportWarnings} />
        }
        <div className="requestTabView"
          style={{ marginTop: '50px', width: '100%', display: 'flex', flexDirection: 'column' }}>
          <TabView activeIndex={activeIndex} onTabChange={(e) => setActiveIndex(e.index)}>

            <TabPanel header="Query Params">
              <div className=""
                style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
                <h2 style={{ marginBottom: '20px' }}>Query Params</h2>
                {
                  currentRequest.query_params?.map((queryParam: QueryParam, index: number) => {
                    return <KeyValueRow key={index} keyProperty={queryParam.key}
                      valueProperty={queryParam.value}
                      active={queryParam.active}
                      keyLabel={"Param Name"} valueLabel={"Param Value"}
                      updateKey={(key: string) => updateQueryParamKey(queryParam, key)}
                      updateValue={(value: string) => updateQueryParamValue(queryParam, value)}
                      updateActive={(active: boolean) => updateQueryParamActive(queryParam, active)}
                      remove={() => removeQueryParam(queryParam)}
                      style={{ marginTop: '20px' }}
                      currentEnvironment={currentEnvironment}
                    />
                  })
                }
                <Button icon={'pi pi-plus'} label={"Query"} onClick={addQueryParam}
                  className={"p-button-sm"}
                  style={{ marginTop: '40px' }} />
              </div>
            </TabPanel>

            <TabPanel header="Headers">
              <div className="headers-block"
                style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
                <h2 style={{ marginBottom: '20px' }}>Headers</h2>
                {
                  currentRequest.headers?.map((header: Header, index: number) => {
                    return <KeyValueRow key={index} keyProperty={header.key}
                      valueProperty={header.value}
                      active={header.active}
                      keyLabel={"Header Name"} valueLabel={"Header Value"}
                      updateKey={(key: string) => updateHeaderKey(header, key)}
                      updateValue={(value: string) => updateHeaderValue(header, value)}
                      updateActive={(active: boolean) => updateHeaderActive(header, active)}
                      remove={() => removeHeader(header)}
                      style={{ marginTop: '20px' }}
                      currentEnvironment={currentEnvironment}
                    />
                  })
                }
                <Button icon={'pi pi-plus'} label={"Header"} onClick={addHeader}
                  className={"p-button-sm"}
                  style={{ margin: '40px 0px' }} />
              </div>
            </TabPanel>

            <TabPanel header="Request Body">
              {/*@TODO*/}
              {/* <RequestBodyComp updateRequest={updateRequest} /> */}
            </TabPanel>

            <TabPanel header="Description">
              <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
                <h2 style={{ marginBottom: '20px' }}>Description</h2>
                <InputTextarea value={currentRequest.description}
                  onChange={(e) => updateDescription(e.target.value)}
                  rows={20}
                  cols={30} autoResize={false} className={'resultText-area'} />
              </div>
            </TabPanel>

          </TabView>
        </div>


        {/*TODO: only show body if request type allows body???*/}


        <div className={"resultText-container"}
          style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', marginTop: '50px' }}>
          <h2>Result</h2>
          <div style={{ width: '100%' }}>
            {
              isSendingRequest && <ProgressSpinner style={{ maxHeight: '80px' }} />
            }
            {
              !isSendingRequest &&
              resultDisplay
            }

          </div>
        </div>
      </div>
    </>
  )
}
