import { useContext, useEffect, useState } from "react";
import { useRequestModelStore } from "../../stores/requestStore";
import { ToastContext } from "../../App";
import { RequestModel, Collection, RequestTree, Environment } from '../../bindings';
import { Button } from "primereact/button";
import { createNewRequestNode } from "../../common/requestUtils";
import { createNewGroupNode } from "../RequestTreeComponent";
import { backend } from '../../rpc';
import { CollectionInfo } from "./CollectionInfo";
import { InputTextarea } from "primereact/inputtextarea";
import { catchError } from "../../common/errorhandling";
import { Accordion, AccordionTab } from "primereact/accordion";
import { WarningCollapsible } from "../WarningCollapsible";
import { getAllRequestsFromTree } from "../../common/treeUtils";

export interface ComponentProps {
}

export function CollectionOverviewComponent(_props: ComponentProps) {

  const toast = useContext(ToastContext);

  const collection = useRequestModelStore((state) => state.currentCollection) as Collection;
  const updateCollection = useRequestModelStore((state) => state.setCurrentCollection);

  const setEnvironments = useRequestModelStore((state) => state.setEnvironments);
  const setCurrentEnvironment = useRequestModelStore((state) => state.setCurrentEnvironment);

  const requestTree = useRequestModelStore((state) => state.requestTree) as RequestTree;
  const [requests, setRequests] = useState<RequestModel[]>([]);

  const workspace = useRequestModelStore((state) => state.workspace)
  const updateWorkspace = useRequestModelStore((state) => state.updateWorkspace)


  useEffect(() => {
    if (requestTree) {
      let allRequests: RequestModel[] = getAllRequestsFromTree(requestTree);
      setRequests(allRequests)
    }
  }, [requestTree])

  useEffect(() => {
    if (collection) {
      backend.loadEnvironments(collection.path).then((environments: Environment[]) => {
        setEnvironments(environments);
        let current_env = environments.find((env: Environment) => env.name == collection.current_env_name);
        if (current_env) {
          setCurrentEnvironment(current_env);
        }
      }).catch(catchError);
    }
  }, [collection?.path]);


  const updateCollectionDescription = (newVal: string) => {
    let newWorkspace = structuredClone(workspace);
    let newCollection = structuredClone(collection);
    newWorkspace.collections = newWorkspace.collections.map((currentCol: Collection) => {
      if (currentCol.path === collection.path) {
        return newCollection;
      } else {
        return currentCol;
      }
    });
    newCollection.description = newVal;
    updateWorkspace(newWorkspace);
    updateCollection(newCollection);
    backend.updateWorkspace(newWorkspace).then(() => {
      // do nothing
    }).catch(catchError);
  }

  const clearImportWarnings = () => {
    onClearWarnings();
  };

  const onClearWarnings = () => {
    let newWorkspace = structuredClone(workspace);
    let newCollection = structuredClone(collection);
    newWorkspace.collections = newWorkspace.collections.map((currentCol: Collection) => {
      if (currentCol.path === collection.path) {
        return newCollection;
      } else {
        return currentCol;
      }
    });
    newCollection.import_warnings = [];
    updateWorkspace(newWorkspace);
    updateCollection(newCollection);
    backend.updateWorkspace(newWorkspace).then(() => {
      toast.showInfo("Cleared import warnings", "");
    }).catch(catchError);
  }

  return (
    <>
      <div className={'fade-in-fast'} style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'flex-start',
        paddingTop: '50px',
        position: 'relative',
      }}>
        <h1 style={{}}>{collection?.name}</h1>
        <div style={{ marginTop: '50px', display: 'flex', justifyContent: 'space-between', width: '100%' }}>
          <div style={{ width: '100%' }} className={'overview-request-list'}>
            {requests && requests.length == 0 &&
              <div>
                <p style={{ textAlign: 'left' }}>You do not have any requests yet in this collection. </p>
                <p style={{ textAlign: 'left' }}>Create a new request or import the
                  requests of an existing collection.</p>

              </div>
            }

            <CollectionInfo collection={collection} displayPathTitle={true} />

            {
              collection.import_warnings.length > 0 &&
              <WarningCollapsible
                collection={collection}
                importWarnings={collection.import_warnings}
                onClearWarnings={clearImportWarnings} />
            }

            <Accordion style={{ marginTop: '50px' }} className={"p-accordion-thin"}>
              <AccordionTab header={"Description"}>
                <div style={{ display: 'flex', alignItems: 'flex-start', flexDirection: 'column' }}>
                  <p style={{ marginTop: '5px', marginBottom: '5px' }}> Describe the purpose of this
                    collection</p>
                  <InputTextarea style={{ minHeight: '300px', width: '100%' }}
                    value={collection.description}
                    onChange={(e) => updateCollectionDescription(e.target.value)}
                    placeholder={"Description"} />
                </div>

              </AccordionTab>
            </Accordion>
            <div style={{
              display: 'flex',
              alignItems: 'flex-start',
              justifyContent: 'flex-start',
              flexDirection: 'column',
              marginTop: '30px'
            }}>

            </div>

            <div style={{ display: 'flex', marginTop: '20px', marginBottom: '20px' }}>
              <div style={{ display: 'flex', flexDirection: 'column' }}>
                <div style={{ marginTop: '20px', display: 'flex' }}>
                  <Button icon={'pi pi-plus'} label={"Create Request"}
                    onClick={() => { createNewRequestNode(requestTree.root, toast, () => { }) }}
                    className={"p-button-sm p-button-text p-button-raised"}
                    style={{}} />
                  <Button icon={'pi pi-plus'} label={"Create Group"}
                    onClick={() => createNewGroupNode(toast, () => {
                    }, collection as Collection, requestTree, requestTree.root, undefined)}
                    className={"p-button-sm p-button-text p-button-raised"}
                    style={{ marginLeft: '10px' }} />
                </div>
              </div>
            </div>
            {/*@TODO {
                            { collection && requestTree &&
                            <RequestTreeComponent withBackgroundColor={false} requestTree={requestTree}
                                                  collection={collection} currentRequest={undefined}/> }
                        } */}
          </div>
        </div>
      </div>
    </>
  )
}
