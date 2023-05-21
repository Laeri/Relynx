import { useRequestModelStore } from "../stores/requestStore";
import { useContext, useEffect, useState } from "react";
// @TODO import {LoadEnvironmentsForCollection, LoadRequestsForSingle,} from "../../wailsjs/go/main/App";
import { Button } from "primereact/button";
//@TODO import {PrimeNode,} from "../common/treeUtils";
import { ToastContext } from "../App";
// @TODO import {catchError, formatParseErrorsMsg} from "../common/errorhandling";
// @TODO import {createNewRequestNode} from "../common/requestUtils";
// @TODO import {createNewGroupNode, RequestTreeComponent} from "./RequestTreeComponent";
// @TODO import {CollectionInfo} from "./CollectionInfo";
import { Collection } from "../bindings";
// @TODO import RequestTreeNode = models.RequestTreeNode;
// @TODO import RequestTree = models.RequestTree;
// @TODO import Environment = models.Environment;
// @TODO import ParseRequestsResult = models.ParseRequestsResult;

interface ComponentProps {
  collection: Collection
}

export function CollectionMenuView(props: ComponentProps) {

  const collection = props.collection;

  /*@TODO const currentRequest = useRequestModelStore((state) => state.currentRequest);

  const setCurrentEnvironment = useRequestModelStore((state) => state.setCurrentEnvironment);

  const updateEnvironments = useRequestModelStore((state) => state.updateEnvironments);

  const requestTree = useRequestModelStore((state) => state.requestTree);
  const updateRequestTree = useRequestModelStore((state) => state.updateRequestTree);
*/
  const [initFinished, setInitFinished] = useState<boolean>(false);

  const toast = useContext(ToastContext);


  useEffect(() => {
    /* 
            LoadRequestsForSingle(collection).then((result: ParseRequestsResult) => {
                if (result.ParseErrors.length > 0) {
                    let allParseErrors = formatParseErrorsMsg(result.ParseErrors);
                    toast.showError("Error loading requests", allParseErrors);
                }
                updateRequestTree(result.RequestTree as RequestTree);
                setInitFinished(true);
            }).catch(catchError(toast));
    
            LoadEnvironmentsForCollection(collection).then((environments: Environment[]) => {
                updateEnvironments(environments);
                let currentEnvironment = undefined;
                if (collection.CurrentEnvName) {
                    currentEnvironment = environments.find((environment: Environment) => environment.Name == collection.CurrentEnvName);
                }
                setCurrentEnvironment(currentEnvironment);
            }).catch(catchError(toast)); */

  }, []); // eslint-disable-line react-hooks/exhaustive-deps


  /* const newRequestNode = (parent: RequestTreeNode, parentPrime?: PrimeNode) => {
      createNewRequestNode(parent, toast, parentPrime);
  } */

  return (
    <div className={"fade-in-fast"} style={{ display: 'flex', flexDirection: 'column' }}>
      {props.collection && initFinished && <>
        {
          <div>
            {/* <CollectionInfo collection={props.collection} displayPathTitle={false}/>
                        <div style={{marginTop: '20px', display: 'flex'}}>
                            <Button icon={'pi pi-plus'} label={"Create Request"}
                                    onClick={() => newRequestNode(requestTree.Root as RequestTreeNode, undefined)}
                                    className={"p-button-sm p-button-text p-button-raised"}
                                    style={{}}/>
                            <Button icon={'pi pi-plus'} label={"Create Group"}
                                    onClick={() => createNewGroupNode(toast, (_node: PrimeNode) => {
                                    }, collection, requestTree, requestTree.Root as RequestTreeNode, undefined)}
                                    className={"p-button-sm p-button-text p-button-raised"}
                                    style={{marginLeft: '10px'}}/>
                        </div>
                        <RequestTreeComponent requestTree={requestTree} collection={collection}
                                              currentRequest={currentRequest} withBackgroundColor={true}/> */}

          </div>
        }
      </>
      }
    </div>
  )
}
