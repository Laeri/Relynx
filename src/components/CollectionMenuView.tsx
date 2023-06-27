import { useRequestModelStore } from "../stores/requestStore";
import { useContext, useEffect, useState } from "react";
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
import { ProgressSpinner } from "primereact/progressspinner";

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

  useEffect(() => {
    backend.loadRequestsForCollection(collection).then((result: LoadRequestsResult) => {
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
    <div className={"fade-in-fast"} style={{ display: 'flex', flexDirection: 'column', height: '100%', flexGrow: 1 }}>
      {props.collection && <>
        {
          <div style={{ height: '100%', display: 'flex', flexDirection: 'column', flexGrow: 1 }}>
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

            {/*@TODO: max height needed otherwise child grows out of parent div and within request tree the list will not be scrolled*/}
            <div style={{ maxHeight: '400px', display: 'flex', flexDirection: 'column', paddingRight: '5px', flexGrow: 1, marginTop: '20px' }}>
              {
                initFinished && <RequestTreeComponent requestTree={requestTree} collection={collection}
                  currentRequest={currentRequest} withBackgroundColor={true} />
              }
              {
                !initFinished && <ProgressSpinner />
              }
            </div>
          </div>
        }
      </>
      }
    </div>
  )
}
