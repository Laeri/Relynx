import { addRequestToRequestTree, PrimeNode } from "./treeUtils";

import { ToastContext } from "../App";
import { NewFError } from "../model/error";
import { catchError, displayAndLogErr } from "./errorhandling";
import { create } from "react-modal-promise";
import { CreateRequestModal } from "../components/modals/CreateRequestModal";
import { newRequestModel } from "../model/request";
import { backend } from '../rpc';
import { useRequestModelStore } from "../stores/requestStore";
import { Collection, RequestModel, RequestTree, RequestTreeNode } from '../bindings';
import { getDefaultRequestName } from "./common";

// @TODO why is parentPrime never used?
export const createNewRequestNode = (parent: RequestTreeNode, toast: ToastContext, _parentPrime?: PrimeNode) => {
  // @TODO: expand parent prime

  const collection = useRequestModelStore.getState().currentCollection as Collection;
  const requestTree = useRequestModelStore.getState().requestTree as RequestTree;
  const updateRequestTree = useRequestModelStore.getState().updateRequestTree;
  const setCurrentRequest = useRequestModelStore.getState().setCurrentRequest;

  if (!parent) {
    let error = NewFError("createNewGroupNode.noParent", "Error during create", "Could not create the new group correctly", "no parent when creating a new group");
    displayAndLogErr(error, toast);
    return
  }

  let requestName = getDefaultRequestName(parent);

  const createRequestModalPromise = create(({ onResolve, onReject, isOpen }) => {
    return <CreateRequestModal requestName={requestName} isOpen={isOpen} onResolve={onResolve} onReject={() => onReject()} />
  });

  createRequestModalPromise().then((requestName?: string) => {
    if (!requestName) {
      return
    }
    let newRequest = newRequestModel({});
    newRequest.name = requestName;
    // normally one request per file but if the parent is a file group there are multiple requests within the same file
    let requestsWithinFile: RequestModel[] = [];

    if (parent.is_file_group) {
      requestsWithinFile = parent.children.filter((child: RequestTreeNode) => child !== undefined)
        .map((child: RequestTreeNode) => child.request as RequestModel);
    }

    backend.addRequestNode(collection, parent, newRequest, requestsWithinFile).then((node: RequestTreeNode) => {
      let [newTree, error] = addRequestToRequestTree(requestTree, parent, node);
      if (error) {
        displayAndLogErr(error, toast);
      }
      updateRequestTree(newTree);
      setCurrentRequest(newRequest);
    }).catch(catchError(toast));

  });
}

// @TODO: stub replace with real function
export function hasInvalidFileBody(request: RequestModel) {
  return false
}
/* export function hasInvalidFileBody(request: RequestModel) {
  return request.body?.BodyType == BodyTypes.BINARY_FILE &&
    (request.body.BinaryFilePath === "" || request.body.BinaryFilePath === undefined || request.body.BinaryFilePath === null)
} */

