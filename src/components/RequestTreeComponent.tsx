import { Tree, TreeDragDropEvent, TreeExpandedKeysType } from "primereact/tree";
import {
  addGroupToRequestTree,
  dragAndDropResult,
  findParent,
  isChildOf, isModelSomewhereWithinGroup,
  PrimeNode, removeNodeFromRequestTree,
  reorderReplace,
  requestTreeToPrimeNodes
} from "../common/treeUtils";
import { useContext, useState } from "react";
import { catchError, displayAndLogErr } from "../common/errorhandling";
import { NewFError } from "../model/error";
import { ToastContext } from "../App";
import { useRequestModelStore } from "../stores/requestStore";
import { ActionDropdown } from "./ActionDropdown";
import { Button } from "primereact/button";
import { createNewRequestNode } from "../common/requestUtils";
import { RequestItemAsButton } from "./RequestItemAsButton";
import { create } from "react-modal-promise";
import { CreateGroupModal } from "./modals/CreateGroupModal";
import { useLocation, useNavigate } from "react-router";
import { Collection, RequestTreeNode, RequestTree, RequestModel, ImportWarning, DragAndDropResult } from '../bindings';
import {backend} from '../rpc';

const updateRequestTree = useRequestModelStore.getState().updateRequestTree;
const setCurrentRequest = useRequestModelStore.getState().setCurrentRequest;

export const createNewGroupNode = (toast: ToastContext, expandNode: (parent: PrimeNode) => void, collection: Collection, requestTree: RequestTree, parent: RequestTreeNode, parentPrime?: PrimeNode) => {
  // @TODO: expand node if closed
  if (!parent) {
    let error = NewFError("createNewGroupNode.noParent", "Error during create", "Could not create the new group correctly", "no parent when creating a new group");
    displayAndLogErr(error, toast);
    return
  }
  const modalPromise = create(({ onResolve, onReject, isOpen }) => {
    return <CreateGroupModal isOpen={isOpen} onResolve={onResolve} onReject={() => onReject()} />
  });

  modalPromise().then((groupName?: string) => {
    if (!groupName) {
      return
    }
    //let newTree = addGroupToRequestTree(requestTree, parent, groupName);
    backend.addGroupNode(collection, parent, groupName).then((requestTreeNode: RequestTreeNode) => {
      let [newTree, error] = addGroupToRequestTree(requestTree, parent, requestTreeNode);
      if (error) {
        displayAndLogErr(error, toast);
      }
      updateRequestTree(newTree);
      if (parentPrime) {
        expandNode(parentPrime);
      }
    }).catch(catchError(toast))
  });
}

export const deleteNode = (toast: ToastContext, collection: Collection, requestTree: RequestTree, node: PrimeNode, currentRequest?: RequestModel) => {
  if (!node) {
    console.error("CollectionMenuView.deleteNode, no node present", parent);
    return
  }

  let treeNode = (node.groupNode ?? node.requestNode) as RequestTreeNode;

  let isGroup = !(node.groupNode === undefined || node.groupNode === null);

  if (!treeNode) {
    let error = NewFError("deleteNode.noTreeNode", "Error during remove", "Could not remove selected element correctly", "no tree node found during delete")
    displayAndLogErr(error, toast);
    return
  }

  // @TODO: pass file_node if parent is a file_node!
  backend.deleteNode(collection, treeNode, null).then(() => {
    // if we delete the current request remove it and change to the collection view
    let currentRequestRemoved = false;
    if (currentRequest) {
      if (treeNode.request?.id) {
        // we removed the node itself
        if (treeNode.request.id == currentRequest.id) {
          currentRequestRemoved = true;
        }
      } else {
        // maybe we removed a group that contains the request
        if (isModelSomewhereWithinGroup(currentRequest.id, treeNode, requestTree)) {
          currentRequestRemoved = true;
        }
      }
    }

    let [newTree, error] = removeNodeFromRequestTree(requestTree, treeNode);
    updateRequestTree(newTree);

    if (currentRequestRemoved) {
      setCurrentRequest(undefined);
    }

    if (error) {
      displayAndLogErr(error, toast);
    } else {
      let successMessage = "Request has been removed";
      if (isGroup) {
        successMessage = "Group has been removed";
      }
      toast.showSuccess(successMessage, "");
    }
  }).catch(catchError(toast));
}


interface ComponentProps {
  requestTree: RequestTree
  collection: Collection

  currentRequest?: RequestModel

  withBackgroundColor: boolean

}

export function RequestTreeComponent(props: ComponentProps) {

  const toast = useContext(ToastContext);
  const updateRequestTree = useRequestModelStore((state) => state.updateRequestTree)
  const [expandedKeys, setExpandedKeys] = useState<TreeExpandedKeysType>({});
  const location = useLocation();

  const expandNode = (primeNode: PrimeNode) => {
    let newExpandedKeys: TreeExpandedKeysType = { ...expandedKeys };
    newExpandedKeys[primeNode.key] = true;
    setExpandedKeys(newExpandedKeys);
  }

  const toggleExpandedKeys = (value: TreeExpandedKeysType) => {
    setExpandedKeys(value);
  }

  const navigate = useNavigate();

  /**
   * Called when drag and drop is used to reorder the tree
   * @param dragNode
   * @param dropNode - undefined if it is the rootnode of the prime tree
   */
  const onReorder = (dragNode: PrimeNode, dropNode: PrimeNode | undefined, dropIndex: number) => {

    // cannot drag node into itself
    if (dragNode.id === dropNode?.id) {
      return
    }

    let dropTreeNode = props.requestTree.root;

    if (dropNode && dropNode.groupNode !== undefined) {
      dropTreeNode = dropNode.groupNode;
    }

    // if we try to drop on a request then insert it before the request in the parent
    if (dropNode && dropNode.requestNode !== undefined) {
      let parentOfDropNode = findParent(props.requestTree, dropNode.requestNode)
      if (parentOfDropNode) {
        dropTreeNode = parentOfDropNode
        dropIndex = parentOfDropNode.children.map((child: RequestTreeNode) => child.id).indexOf(dropNode.requestNode.id)
      }
    }

    if (!dropTreeNode) {
      displayAndLogErr(NewFError("onReorder.noDropTreeNode", "Could not reorder elements", "", "drop tree node not present in onReorder"), toast)
      return
    }
    let dragTreeNode = (dragNode.groupNode ?? dragNode.requestNode) as RequestTreeNode
    if (!dragTreeNode) {
      displayAndLogErr(NewFError("onReorder.noDragTreeNode", "Could not reorder elements", "", "drag tree node not present in onReorder"), toast)
    }

    // if we reorder a node within a group then we do not need a backend call
    if (isChildOf(props.requestTree, dragTreeNode, dropTreeNode)) {
      backend.reorderNodesWithinParent(props.collection, dragTreeNode, dropTreeNode, dropIndex).then((newDropNode: RequestTreeNode) => {
        let [newTree, error] = reorderReplace(props.requestTree, newDropNode);
        if (error) {
          displayAndLogErr(error, toast);
        }
        updateRequestTree(newTree);
      }).catch(catchError(toast));

    } else {
      let dragNodeParent = findParent(props.requestTree, dragTreeNode)
      if (!dragNodeParent) {
        displayAndLogErr(NewFError("onReorder.noParentForDragTreeNode", "Could not reorder elements", "", "Could not find parent of dragTreeNode for DragAndDrop argument"), toast)
        return
      }
      backend.dragAndDrop(props.collection, dragNodeParent, dragTreeNode, dropTreeNode, dropIndex).then((ddResult: DragAndDropResult) => {
        if (!ddResult.new_drop_node) {
          return
        }
        let [newTree, error] = dragAndDropResult(props.requestTree, dragTreeNode, ddResult, dropIndex);
        if (error) {
          displayAndLogErr(error, toast);
        }
        updateRequestTree(newTree);
        if (dropNode) {
          expandNode(dropNode);
        }
      }).catch(catchError(toast));
    }


  }

  const onPrimeTreeNodeClicked = (primeNode: PrimeNode) => {
    if (primeNode.requestNode?.request) {
      setCurrentRequest(primeNode.requestNode.request);
      let options = { replace: true };
      if (location.pathname == '/collection') {
        options.replace = false;
      }
      navigate('/collection/request', options);
    }
    // TODO: update server side
  }

  const nodeTemplate = (node: any, _options: any) => {

    let primeNode = node as PrimeNode
    if (primeNode.isFolder) {
      // we have a group
      return (
        <div style={{ width: '100%', display: 'flex', alignItems: 'center', maxWidth: '100%' }}>
          <div className={'p-button-raised p-button-text'}
            style={{ flexGrow: 1, display: 'flex', justifyContent: 'flex-start', alignItems: 'center' }}>
            {
              <span className={`pi ${primeNode.groupNode?.is_file_group ? 'pi-file' : 'pi-folder'}`} />
            }
            {/*TODO: maybe add info icon that this is a file group? */}
            <h4 style={{
              marginLeft: '10px',
              display: 'flex',
              alignItems: 'center',
              cursor: 'pointer'

            }}>{primeNode.label} {primeNode.groupNode?.is_file_group ? '(File)' : ''}</h4>
          </div>

          <ActionDropdown styles={{ flexGrow: 1, marginRight: '3px' }}>
            <Button icon={'pi pi-plus'} className={'p-button p-button-text'}
              label={"Create Request"}
              onClick={() => createNewRequestNode(node.groupNode, node)} />
            {
              !node.groupNode.IsFileGroup &&
              <Button icon={'pi pi-plus'} className={'p-button p-button-text'}
                label={"Create Group"}
                onClick={() => createNewGroupNode(toast, (_node: PrimeNode) => {
                }, props.collection, props.requestTree, node.groupNode, node)} />
            }

            <Button icon={'pi pi-trash'} className={'p-button p-button-text'}
              label={"Delete Group"}
              onClick={() => deleteNode(toast, props.collection, props.requestTree, node, props.currentRequest)} />
          </ActionDropdown>
        </div>
      )

    } else {
      // we have a request
      return (
        <div style={{ width: '100%', display: 'flex', alignItems: 'center' }}>
          <RequestItemAsButton onClick={() => onPrimeTreeNodeClicked(primeNode)} label={primeNode.label}
            requestModel={primeNode.requestNode?.request as RequestModel}
            requestId={primeNode.key}
            importWarnings={props.collection.import_warnings.filter((importWarning: ImportWarning) => {
              let requestModel = primeNode.requestNode?.request as RequestModel
              return importWarning.rest_file_path == requestModel.rest_file_path && importWarning.request_name == requestModel.name
            })}
            highlighted={props.currentRequest?.id == primeNode.key} />
          <ActionDropdown styles={{ flexGrow: 1, marginRight: '3px' }}>
            <Button icon={'pi pi-trash'} className={' p-button-text'}
              label={"Delete Request"}
              onClick={() => deleteNode(toast, props.collection, props.requestTree, node)} />
          </ActionDropdown>

        </div>
      )
    }
  }

  return (
    <Tree className={"p-tree-no-background-color"} value={requestTreeToPrimeNodes(props.requestTree)}
      nodeTemplate={nodeTemplate}
      dragdropScope="collectionTree"
      expandedKeys={expandedKeys}
      filter={true}
      onToggle={(e) => toggleExpandedKeys(e.value)}
      onDragDrop={
        (event: TreeDragDropEvent) => {
          // @ts-ignore, ignore for now, created an issue on GitHub and will be fixed in the next release
          let dropNode: any = event.dropNode;
          onReorder(event.dragNode as PrimeNode, dropNode, event.dropIndex)
        }
      }
      style={{ border: 'none', marginTop: '10px' }} />
  )
}
