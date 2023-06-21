import { useRequestModelStore } from "../stores/requestStore";
import React, { useContext, useEffect, useState } from "react";
import { Button } from "primereact/button";
import { PrimeNode, } from "../common/treeUtils";
import { ToastContext } from "../App";
import { createNewRequestNode } from "../common/requestUtils";
import { createNewGroupNode, RequestTreeComponent } from "./RequestTreeComponent";
import { CollectionInfo } from "./CollectionInfo";
import { Collection, RequestTree, RequestTreeNode } from "../bindings";
//
import { backend } from '../rpc';
import { LoadRequestsResult } from "../bindings";
import { catchError, formatParseErrorsMsg } from "../common/errorhandling";

interface ComponentProps {
  collection: Collection
}

export function CollectionMenuView(props: ComponentProps) {

  const collection = props.collection;

  const requestTree = useRequestModelStore((state) => state.requestTree) as RequestTree;// @TODO check that request tree is not null

  const currentRequest = useRequestModelStore((state) => state.currentRequest);

  const updateRequestTree = useRequestModelStore((state) => state.updateRequestTree);
  const [initFinished, setInitFinished] = useState<boolean>(false);

  const toast = useContext(ToastContext);

  /* 
*  
 LoadEnvironmentsForCollection(collection).then((environments: Environment[]) => {
     updateEnvironments(environments);
     let currentEnvironment = undefined;
     if (collection.CurrentEnvName) {
         currentEnvironment = environments.find((environment: Environment) => environment.Name == collection.CurrentEnvName);
     }
     setCurrentEnvironment(currentEnvironment);
 }).catch(catchError(toast));  
*/
  useEffect(() => {
    backend.loadRequestsForCollection(collection).then((result: LoadRequestsResult) => {

      console.log('RESULT: ', result);
      if (result.errs.length > 0) {
        let allParseErrors = formatParseErrorsMsg(result.errs);
        toast.showError("Error loading requests", allParseErrors);
      }
      updateRequestTree(result.request_tree);
      setInitFinished(true);
    }).catch(catchError(toast));

  }, []);

  // eslint-disable-line react-hooks/exhaustive-deps


  const newRequestNode = (parent: RequestTreeNode, parentPrime?: PrimeNode) => {
    createNewRequestNode(parent, toast, (_nodeId: string) => {
      // ignore, we do not want to expand as the request is added to the top level node (root)
    });
  }

  return (
    <div className={"fade-in-fast"} style={{ display: 'flex', flexDirection: 'column' }}>
      {props.collection && <>
        {
          <div>
            <CollectionInfo collection={props.collection} displayPathTitle={false} />
            <div style={{ marginTop: '20px', display: 'flex' }}>
              <Button icon={'pi pi-plus'} label={"Create Request"}
                onClick={() => newRequestNode(requestTree.root, undefined)}
                className={"p-button-sm p-button-text p-button-raised"}
                style={{}} />
              <Button icon={'pi pi-plus'} label={"Create Group"}
                onClick={() => createNewGroupNode(toast, (_node: string) => {
                }, collection, requestTree, requestTree.root, undefined)}
                className={"p-button-sm p-button-text p-button-raised"}
                style={{ marginLeft: '10px' }} />
            </div>
            {
              initFinished && <RequestTreeComponent requestTree={requestTree} collection={collection}
                currentRequest={currentRequest} withBackgroundColor={true} />
            }

          </div>
        }
      </>
      }
    </div>
  )
}
