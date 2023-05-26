import { RequestModel } from '../bindings';
/* @TODO import {addRequestToRequestTree, PrimeNode} from "./treeUtils";
*
import {ToastContext} from "../App";
import {NewFError} from "../models/Error";
import {catchError, displayAndLogErr} from "./errorhandling";
import {create} from "react-modal-promise";
import {CreateRequestModal} from "../components/modals/CreateRequestModal";
import {BodyTypes, newRequestModel} from "../models/Request";
import {AddRequestNode} from "../../wailsjs/go/main/App";
import {useRequestModelStore} from "../stores/requestStore";
import React from "react";
import RequestTreeNode = models.RequestTreeNode;
import RequestModel = models.RequestModel;
import AddRequestNodeResult = models.AddRequestNodeResult;
import {RequestModel} from '../bindings';

export const createNewRequestNode = (parent: RequestTreeNode, toast: ToastContext, parentPrime?: PrimeNode) => {
    // @TODO: expand parent prime

    const collection = useRequestModelStore.getState().currentCollection;
    const requestTree = useRequestModelStore.getState().requestTree;
    const updateRequestTree = useRequestModelStore.getState().updateRequestTree;
    const setCurrentRequest = useRequestModelStore.getState().setCurrentRequest;

    if (!parent) {
        let error = NewFError("createNewGroupNode.noParent", "Error during create", "Could not create the new group correctly", "no parent when creating a new group");
        displayAndLogErr(error, toast);
        return
    }

    const createRequestModalPromise = create(({onResolve, onReject, isOpen}) => {
        return <CreateRequestModal isOpen={isOpen} onResolve={onResolve} onReject={() => onReject()}/>
    });

    createRequestModalPromise().then((requestName?: string) => {
        if (!requestName) {
            return
        }
        let newRequest = newRequestModel();
        newRequest.Name = requestName;
        // normally one request per file but if the parent is a file group there are multiple requests within the same file
        let requestsWithinFile: RequestModel[] = []

        if (parent.IsFileGroup) {
            requestsWithinFile = parent.Children.filter((child: RequestTreeNode) => child !== undefined)
                .map((child: RequestTreeNode) => child.RequestModel as RequestModel)
        }

        AddRequestNode(collection, parent, newRequest, requestsWithinFile).then((result: AddRequestNodeResult) => {
            if (!result.RequestTreeNode) {
                return
            }
            let [newTree, error] = addRequestToRequestTree(requestTree, parent, result.RequestTreeNode);
            if (error) {
                displayAndLogErr(error, toast);
            }
            updateRequestTree(newTree);
            setCurrentRequest(newRequest);
        }).catch(catchError(toast));

    });
}
 
export function hasInvalidFileBody(request: RequestModel) {
    return request.RequestBody?.BodyType == BodyTypes.BINARY_FILE &&
        (request.RequestBody.BinaryFilePath === "" || request.RequestBody.BinaryFilePath === undefined || request.RequestBody.BinaryFilePath === null)
}
*/
export function hasInvalidFileBody(_request: RequestModel): boolean {
  throw new Error("@TODO requestUtils.tsx::hasInvalidFileBody");
}
