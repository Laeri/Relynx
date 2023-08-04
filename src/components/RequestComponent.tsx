import { useContext, useEffect, useState } from "react";
import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";
import { Dropdown } from "primereact/dropdown";
import { KeyValueRow } from "./KeyValueRow";
import { InputTextarea } from "primereact/inputtextarea";
import { TabPanel, TabView } from "primereact/tabview";
import { useRequestModelStore } from "../stores/requestStore";
import { backend } from '../rpc';
import { RequestModel, QueryParam, Header, Collection, ImportWarning, RunRequestCommand, RequestResult, EnvironmentVariable, EnvironmentSecret, HttpMethod } from '../bindings';
import { ToastContext } from "../App";
import { catchError, catchErrorWithTitle } from "../common/errorhandling";
import { Message } from "primereact/message";
import { hasInvalidFileBody } from "../common/requestUtils";
import { updatedRequestModel, newRequestHeader } from '../model/model';
import { getAllRequestsFromTree } from "../common/treeUtils";
import { changeRequestUrlParams, changeUrlParams, extractQueryParamsFromUrl, HTTP_METHODS, isCustomMethod } from "../model/request";
import { RequestImportMessages } from "./RequestImportMessages";
import { Accordion, AccordionTab } from "primereact/accordion";
import { RequestSettingsComponent } from "./RequestSettingsComponent";
import { CancellationToken } from "../model/error";
import { Dialog } from "primereact/dialog";
import { ResultDisplay } from "./ResultDisplay";
import { SendRequestButton } from "./SendRequestButton";
import { RequestBodyComp } from "./body/RequestBodyComp";
import { QueryParams } from "./QueryParams";
import { CookieJarComponent } from "./CookieJarComponent";
import { openEditRequestNameModal } from "../common/modal";
import { Headers } from "./Headers";

interface ComponentProps {
}

export function RequestComponent(_props: ComponentProps) {

  const currentRequest = useRequestModelStore((state) => state.currentRequest as RequestModel);
  const storeUpdateRequestAndTree = useRequestModelStore((state) => state.storeUpdateRequestAndTree);

  const requestResult = useRequestModelStore((state) => state.requestResult);
  const setRequestResult = useRequestModelStore((state) => state.setRequestResult);
  const workspace = useRequestModelStore((state) => state.workspace);
  const updateWorkspace = useRequestModelStore((state) => state.updateWorkspace);

  // @TODO const updateRequestResult = useRequestModelStore((state) => state.updateRequestResult);
  const storeClearResult = useRequestModelStore((state) => state.clearRequestResult);

  const currentCollection = useRequestModelStore((state) => state.currentCollection as Collection);

  const currentEnvironment = useRequestModelStore((state) => state.currentEnvironment);

  const requestTree = useRequestModelStore((state) => state.requestTree);

  const [activeIndex, setActiveIndex] = useState<number>(0);

  const toast = useContext(ToastContext);

  const [isSendingRequest, setIsSendingRequest] = useState<boolean>(false);

  const [importWarnings, setImportWarnings] = useState<ImportWarning[]>([]);

  const [customRequestType, setCustomRequestType] = useState<string>("CUSTOM");

  const [cancelToken, setCancelToken] = useState<CancellationToken>({ cancelled: false });

  const [resultVisible, setResultVisible] = useState<boolean>(false);

  const [resultHistory, setResultHistory] = useState<RequestResult[]>([]);

  const [url, setUrl] = useState<string>();

  useEffect(() => {
    setIsSendingRequest(false);
    setCancelToken({ cancelled: true });
    setResultHistory([]);
    setRequestResult(undefined);
    //scrollMainToTop();
    if (!currentRequest) {
      return
    }

    let importWarnings: ImportWarning[] = (currentCollection as Collection).import_warnings.filter((import_warning: ImportWarning) => {
      return import_warning.rest_file_path === currentRequest.rest_file_path;
    });
    setImportWarnings(importWarnings);
    // @TODO: setOverwriteResponseFile(currentRequest.)
    setUrl(currentRequest.url);
    let queryParams = extractQueryParamsFromUrl(currentRequest);
    let newRequest = updatedRequestModel(currentRequest, { query_params: queryParams });
    updateRequest(newRequest);

  }, [currentRequest?.id]);


  // send request data to backend and perform libcurl request
  function doRequest() {
    let backendRequest: RunRequestCommand = {
      request: currentRequest,
      collection: currentCollection,
      environment: null
    }

    // filter out variables with duplicated key names (keep the first one)
    if (currentEnvironment) {
      let backendEnvironment = structuredClone(currentEnvironment);

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
    if (requestResult) {
      let newHistory = [requestResult, ...resultHistory];
      setResultHistory(newHistory);
    }


    // @TODO: cancel request
    let newCancelToken = { cancelled: false };
    setCancelToken(newCancelToken);
    backend.runRequest(backendRequest, newCancelToken).then((result: RequestResult) => {
      if (newCancelToken.cancelled) {
        return
      }
      setRequestResult(result);

      showResult();

      result.warnings.forEach((warning: string) => {
        toast.showWarn('', warning, 30000);
      });
    }).catch((err) => catchErrorWithTitle("There was an error sending the request", err)).finally(() => {
      setIsSendingRequest(false);
    });
  }

  function editRequestName() {
    openEditRequestNameModal(currentRequest, currentCollection, (newName: string) => {
      console.log('h1');
      let newRequest = updatedRequestModel(currentRequest, { name: newName });
      console.log('h2');
      return updateRequest(newRequest);
    });
  }

  function updateRequest(newRequest: RequestModel): Promise<void> {
    console.log('h3');
    if (!currentCollection || !currentRequest) {
      return Promise.resolve();
    }
    let allRequests: RequestModel[] = [];
    if (requestTree) {
      allRequests = getAllRequestsFromTree(requestTree);
    }

    // find all requests with the same path which means they are in the same file
    // @SPEED, we have to parse the tree everytime we do this
    let requestsInSameFile = allRequests.filter((request: RequestModel) => request.rest_file_path == newRequest.rest_file_path || request.id == newRequest.id);

    // replace the new request within our requests here
    requestsInSameFile = requestsInSameFile.map((request: RequestModel) => {
      if (request.id === newRequest.id) {
        return newRequest
      } else {
        return request
      }
    });

    setUrl(newRequest.url);
    console.log('before call');
    return new Promise<void>((resolve: any, _reject: any) => {
      backend.saveRequest(requestsInSameFile, currentCollection, currentRequest.name).then((newPath: string) => {
        newRequest.rest_file_path = newPath;
        console.log('got reslut from backend a ok');
        storeUpdateRequestAndTree(newRequest)
        resolve();
      }).catch(catchError);
    });
  }

  function updateUrl(url: string) {
    let newRequest = updatedRequestModel(currentRequest, { url: url });
    let queryParams = extractQueryParamsFromUrl(newRequest);
    newRequest.query_params = queryParams;
    setUrl(url);
    updateRequest(newRequest);
  }

  function updateDescription(description: string) {
    let newRequest = updatedRequestModel(currentRequest, { description: description });
    updateRequest(newRequest);
  }

  function updateRequestType(method: string) {
    let newRequest;
    if (method == "CUSTOM") {
      newRequest = updatedRequestModel(currentRequest, { method: { CUSTOM: customRequestType } });
    } else {
      newRequest = updatedRequestModel(currentRequest, { method: method as HttpMethod });
    }
    updateRequest(newRequest);
  }

  function updateCustomMethod(value: string) {
    setCustomRequestType(value);
    let newRequest = updatedRequestModel(currentRequest, { method: { CUSTOM: value } });
    updateRequest(newRequest);
  }

  function updateQueryParam(oldParam: QueryParam, newParam: QueryParam) {
    let newQueryParams = [...currentRequest.query_params];
    let index = newQueryParams.indexOf(oldParam);
    newQueryParams[index] = newParam;

    let newRequest = updatedRequestModel(currentRequest, {});

    changeRequestUrlParams(newRequest, oldParam, newParam);
    updateRequest(newRequest);
  }


  function addQueryParam() {
    // let newRequestModel: RequestModel = updatedRequestModel(currentRequest, {
    //   query_params: [...currentRequest.query_params, newQueryParam(undefined)]
    // });
    let newUrl = changeUrlParams(currentRequest.url, undefined, { key: "", value: "", active: true })
    let newRequest = updatedRequestModel(currentRequest, { url: newUrl });
    newRequest.query_params = extractQueryParamsFromUrl(newRequest);
    updateRequest(newRequest);
  }

  function removeQueryParam(queryParam: QueryParam) {

    let newUrl = changeUrlParams(currentRequest.url, queryParam, undefined);
    let newParams = currentRequest.query_params.filter((param: QueryParam) => param.key !== queryParam.key);
    let newRequestModel: RequestModel = updatedRequestModel(currentRequest, { url: newUrl, query_params: newParams }
    );
    updateRequest(newRequestModel);
  }



  const requestTypeOptions: { name: string, value: string }[] = Object.entries(HTTP_METHODS).map(([key, value]) => {
    return { name: key, value: value as string };
  });

  requestTypeOptions.push({ name: "CUSTOM", value: "CUSTOM" })

  const clearResult = (result: RequestResult) => {
    if (requestResult == result) {
      storeClearResult();
    }
    let resultIndex = resultHistory.findIndex((current: RequestResult) => current.id === result.id);
    if (resultIndex !== -1) {
      let newHistory = [...resultHistory];
      newHistory.splice(resultIndex, 1);
      setResultHistory(newHistory);
    }
  }


  // Result needs to be memoized because if it is updated frequently it will freeze inputs

  const clearImportWarnings = () => {
    if (!currentCollection) {
      return
    }
    let newWorkspace = structuredClone(workspace);
    let newCollection = structuredClone(currentCollection);
    newCollection.import_warnings = [];
    let new_index = newWorkspace.collections.findIndex((currentCol: Collection) => {
      currentCol.path == currentCollection.path;
    });
    newWorkspace.collections[new_index] = newCollection;
    updateWorkspace(newWorkspace);
    backend.updateWorkspace(newWorkspace).then(() => {
      // do nothing
    }).catch(catchError);
  }

  const cancelCurrentRequest = () => {
    cancelToken.cancelled = true;
    setIsSendingRequest(false);
    // @TODO: cancel in backend as well... backend.cancelCurlRequest().then(()=>{}).catch(catchError);
  }

  const showResult = () => {
    // @ts-ignore
    setResultVisible(true);
  }

  return (
    <div className={'fade-in-fast'} style={{ position: 'relative', display: 'flex', flexDirection: 'column', alignItems: 'flex-start', marginBottom: '30px' }}>

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
        marginTop: '50px',
        marginBottom: '20px'
      }}>
        <div style={{ width: '100%', display: 'flex', alignItems: 'center' }}>
          <h2>{currentRequest.name}</h2>
          <Button className="ml-2" size="small" text={true} tooltip="Edit Name" icon={"pi pi-pencil"} onClick={editRequestName} />
        </div>
      </div>
      <div style={{ display: 'flex', flexDirection: 'row', marginTop: '30px', maxWidth: '100%', width: '100%' }}>
        <div style={{ height: '100%' }}>
          <Dropdown style={{ height: '100%' }} disabled={isSendingRequest} optionLabel="name" value={isCustomMethod(currentRequest.method) ? "CUSTOM" : currentRequest.method} options={requestTypeOptions}
            onChange={(e) => updateRequestType(e.value)} />
        </div>

        <InputText value={url} onChange={(e) => updateUrl(e.target.value)} placeholder={"Url"}
          style={{ maxWidth: '300px', marginLeft: '20px', flexGrow: 1 }} disabled={isSendingRequest} />
        <SendRequestButton style={{ marginLeft: '20px' }} isSendingRequest={isSendingRequest} cancelRequest={cancelCurrentRequest} doRequest={doRequest} disabled={isSendingRequest || currentRequest.url == ""} />
      </div>

      {
        isCustomMethod(currentRequest.method) &&
        <div style={{ display: 'flex', marginTop: '10px' }}>
          <InputText value={(currentRequest.method as { CUSTOM: string }).CUSTOM} onChange={(e) => updateCustomMethod(e.target.value)} placeholder={"Custom Method"}
            style={{ maxWidth: '150px' }} disabled={isSendingRequest} />

        </div>
      }

      <Button label="Show Result" style={{ marginTop: '20px', display: 'flex', alignItems: 'center' }} icon={isSendingRequest ? "pi pi-spin pi-spinner" : "pi pi-chevron-down"} onClick={showResult} text />
      <Dialog header="Result" closeOnEscape={true} maximizable={true} dismissableMask={true} visible={resultVisible} style={{ width: '70vw' }} onHide={() => setResultVisible(false)}>
        <ResultDisplay sendRequest={doRequest} cancelRequest={cancelCurrentRequest} requestSendDisabled={isSendingRequest || currentRequest.url == ""} isSendingRequest={isSendingRequest} clearResult={clearResult} requestResult={requestResult} resultHistory={resultHistory} />
      </Dialog>

      {
        importWarnings.length > 0 &&
        <Accordion style={{ marginTop: '30px' }} className={"p-accordion-thin"}>
          <AccordionTab
            className={"wide-accordion-header"}
            header={
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <div>
                  <i className="pi pi-exclamation-triangle mr-2 color-warn"></i>
                  <span
                    className="vertical-align-middle color-warn">Import Problems</span>
                </div>
              </div>
            }
          >
            <RequestImportMessages absolutePath={currentRequest.rest_file_path} messages={importWarnings} collection={currentCollection} />
          </AccordionTab>
        </Accordion>
      }
      <div className="requestTabView"
        style={{ marginTop: '50px', width: '100%', display: 'flex', flexDirection: 'column' }}>
        <TabView activeIndex={activeIndex} onTabChange={(e) => setActiveIndex(e.index)}>

          <TabPanel header="Query Params" >
            <div className="headers-block"
              style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
              <h2 style={{ marginBottom: '20px' }}>Query Params</h2>
              <QueryParams
                queryParams={currentRequest.query_params}
                addQueryParam={addQueryParam}
                removeQueryParam={removeQueryParam}
                updateQueryParam={updateQueryParam}
                currentEnvironment={currentEnvironment}
              />
            </div>
          </TabPanel>

          <TabPanel header="Headers">
            <Headers updateRequest={updateRequest} request={currentRequest} />
          </TabPanel>

          <TabPanel header={"Body"}>
            <RequestBodyComp updateRequest={updateRequest} request={currentRequest} environment={currentEnvironment} />
          </TabPanel>

          <TabPanel header="Description">
            <div className="headers-block" style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
              <h2>Description</h2>
              <p style={{ marginTop: '20px' }}>Describe the current request</p>
              <InputTextarea style={{ marginTop: '20px' }} value={currentRequest.description}
                onChange={(e) => updateDescription(e.target.value)}
                rows={20}
                cols={30} autoResize={false} className={'resultText-area'} />
            </div>
          </TabPanel>

          <TabPanel header="RequestSettings">
            <RequestSettingsComponent request={currentRequest} updateRequest={updateRequest} collection={currentCollection} />
          </TabPanel>

        </TabView>
      </div>
    </div>
  )
}
