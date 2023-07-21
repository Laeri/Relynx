import { RequestTree, RequestTreeNode, RequestModel, DragAndDropResult } from '../bindings';
import { newUUID, UUID } from "../model/request";
import { FError } from "./errorhandling";
import { NewFError } from "../model/error";
import { backend } from '../rpc';


export function requestTreeToPrimeNodes(requestTree: RequestTree): PrimeNode[] {

  if (!requestTree || !requestTree.root) {
    return [];
  }

  let parentMap = new Map<RequestTreeNode, PrimeNode>();

  let primeRoot = newPrimeNode();
  primeRoot.id = requestTree.root?.id;

  let nodesQueue: RequestTreeNode[] = [];
  if (requestTree.root) {
    nodesQueue.push(...requestTree.root.children);
  }


  while (nodesQueue.length > 0) {
    let current = nodesQueue.shift();

    if (!current) {
      continue
    }

    let parentNode: PrimeNode = primeRoot;
    if (parentMap.has(current)) {
      // @ts-ignore, already checked above
      parentNode = parentMap.get(current);
    }

    let primeNode: PrimeNode = newPrimeNode();
    primeNode.id = current.id;
    parentNode.children = [...parentNode.children, primeNode];

    if (current.request) {
      primeNode.key = current.id;
      primeNode.label = current.request.name;
      primeNode.requestNode = current;
      primeNode.isFolder = false;
    } else {
      primeNode.groupNode = current;
      primeNode.key = current.id;
      primeNode.label = current.name;
      primeNode.isFolder = true;
      // we have a group
      current.children.forEach((requestTreeNode: RequestTreeNode) => {
        parentMap.set(requestTreeNode, primeNode);
      })
      nodesQueue = [...nodesQueue, ...current.children];
    }
  }

  // we do not require the root node here
  return primeRoot.children;
}

export function newRequestTree(): RequestTree {
  return {
    root: newRequestTreeNode()
  }
}

/**
 * Update the request tree when the request model changed. Match RequestModel with their id within
 * the tree and return a tree copy with the new RequestModel inserted where the old one was
 * @param requestTree
 * @param request
 */
export function getUpdateRequestTreeWithRequest(requestTree: RequestTree, request: RequestModel): RequestTree {
  if (!requestTree.root) {
    console.error("getUpdateRequestTreeWithRequest: tree with no root encountered!");
    return requestTree;
  }
  // @TODO @SPEED, we re create the full tree
  let newTree = newRequestTree();
  newTree.root = getUpdatedNodeWithRequest(requestTree.root, request);

  return newTree;
}

/**
 * Helper function for creating a new updated copy of a RequestTree
 * @param node
 * @param request
 */
function getUpdatedNodeWithRequest(node: RequestTreeNode, request: RequestModel): RequestTreeNode {
  if (node.request) {
    let nodeModelId = node.request.id;
    let requestId = request.id;

    if (nodeModelId.trim() === requestId.trim()) {
      let newNode = newRequestTreeNode();
      newNode.name = request.name;
      newNode.filepath = node.filepath;
      newNode.request = request;
      newNode.children = [...node.children];
      return newNode;
    } else {
      return node;
    }
  } else {
    node.children = node.children.map((child: RequestTreeNode) => {
      return getUpdatedNodeWithRequest(child, request);
    })
    return node;
  }
}

/**
 * Interface for a node with in the prime react tree
 * see https://www.primefaces.org/primereact/tree/
 */
export interface PrimeNode {
  id: string,
  key: string,
  label: string
  children: PrimeNode[],
  requestNode: RequestTreeNode | undefined,
  groupNode: RequestTreeNode | undefined
  isFolder: boolean
}

/**
 * Transform our request tree model from the prime react tree model.
 * This is only used when reordering nodes using drag and drop
 * see https://www.primefaces.org/primereact/tree/
 * @param primesNodes
 */
export function buildRequestTreeFromPrimeNodes(primesNodes: PrimeNode[]): RequestTree {
  let tree = newRequestTree();
  let root = newRequestTreeNode();
  tree.root = root;
  root.children = primesNodes.map((primeNode: PrimeNode) => primeNodeToRequestTreeNode(primeNode));
  return tree;
}

/**
 * Helper method to convert a primeNode back to a request tree node
 * @param primeNode
 */
function primeNodeToRequestTreeNode(primeNode: PrimeNode): RequestTreeNode {
  if (!primeNode.requestNode) {
    let newGroupNode = newRequestTreeNode();
    newGroupNode.name = primeNode.label;
    newGroupNode.children = primeNode.children.map((primeChild: PrimeNode) => primeNodeToRequestTreeNode(primeChild));
    return newGroupNode;
  } else {
    return structuredClone(primeNode.requestNode);
  }
}

// Manipulation

export function cloneTree(requestTree: RequestTree): RequestTree {
  return structuredClone(requestTree);
}

/**
 * Second return value is error message
 * @param requestTree
 * @param parent
 * @param requestTreeNode
 */
export function addRequestToRequestTree(requestTree: RequestTree, parent: RequestTreeNode, requestTreeNode: RequestTreeNode): [RequestTree, FError] {

  let newTree = cloneTree(requestTree);

  let newParent = findNode(newTree, parent.id);
  if (!newParent) {
    return [newTree, NewFError("addRequestToRequestTree.noParent", "Error during add", "Could not add request correctly", "Could not add request correctly")];
  }
  newParent.children.unshift(requestTreeNode);
  return [newTree, undefined]
}

export function renameGroupNode(requestTree: RequestTree, groupNodeId: string, { newName, newPath }: { newName: string, newPath: string }): [RequestTree, FError] {

  let newTree = cloneTree(requestTree);

  let groupNode = findNode(newTree, groupNodeId);
  if (!groupNode) {
    return [newTree, NewFError("addRequestToRequestTree.noParent", "Error during rename", "Could not rename group correctly", "Could not rename group correctly")];
  }
  let oldPath = groupNode.filepath;
  groupNode.name = newName;
  groupNode.filepath = newPath;
  fixRenamePathsOfChildren(groupNode, oldPath);

  return [newTree, undefined]
}

export function fixRenamePathsOfChildren(parent: RequestTreeNode, oldPath: string) {
  let nodes = [...parent.children];
  while (nodes.length > 0) {
    let currentNode = nodes.pop();
    if (!currentNode) {
      continue
    }
    currentNode.filepath = currentNode.filepath.replace(oldPath, parent.filepath);
    if (currentNode?.request) {
      currentNode.request.rest_file_path = currentNode.request.rest_file_path.replace(oldPath, parent.filepath);
    }
    if (currentNode?.children) {
      nodes.push(...currentNode.children);
    }
  }
}

export function addGroupToRequestTree(requestTree: RequestTree, parent: RequestTreeNode, newGroup: RequestTreeNode): [RequestTree, FError] {
  let newTree = cloneTree(requestTree);
  let newParent = findNode(newTree, parent.id);
  if (!newParent) {
    return [newTree, NewFError("addGroupToRequestTree.noParent", "Error during add", "Could not add group correctly", "Could not add group correctly")];
  }
  newParent.children.unshift(newGroup);
  return [newTree, undefined];
}

export function findParent(requestTree: RequestTree, node: RequestTreeNode): RequestTreeNode | undefined {
  return findParentById(requestTree, node.id);
}

export function findParentById(requestTree: RequestTree, nodeId: string): RequestTreeNode | undefined {
  if (!requestTree.root) {
    return undefined;
  }
  let nodes = [requestTree.root]
  while (nodes.length > 0) {
    let current = nodes.shift();
    if (!current) {
      continue
    }
    if (current.children.length > 0) {
      nodes.push(...current.children);
      if (current.children.some((childNode: RequestTreeNode) => childNode.id == nodeId)) {
        return current;
      }
    }
  }
  return undefined
}

export function isChildOf(requestTree: RequestTree, child: RequestTreeNode, parent: RequestTreeNode): boolean {
  // TODO maybe have only an 'rTreeNode' within the prime node and test on there if it is a group or request node!!!!
  let dragRParent = findParent(requestTree, child);

  if (!dragRParent) {
    console.error("Neither first nor second parent");
    return false;
  }

  return dragRParent?.id === parent.id;
}

export function findNode(requestTree: RequestTree, id: UUID, checkModelId: boolean = false) {
  if (!requestTree.root) {
    return undefined;
  }
  let nodes = [requestTree.root];
  while (nodes.length > 0) {
    let current = nodes.shift();
    if (!current) {
      continue;
    }

    if (checkModelId) {
      if (current.request?.id == id) {
        return current
      }
    } else {
      if (current.id == id) {
        return current;
      }

    }
    if (current.children.length > 0) {
      nodes.push(...current.children);
    }
  }
  return undefined;
}


export function removeNodeFromRequestTree(requestTree: RequestTree, node: RequestTreeNode): [RequestTree, FError] {
  let newTree = cloneTree(requestTree);
  let oldParent = findParent(requestTree, node);
  if (!oldParent) {
    return [newTree, NewFError("removeNodeFromRequestTree.noOldParent", "Error during remove", "Could not remove the selected request or group correctly", "no old parent found in remove")]
  }
  let newParent = findNode(newTree, oldParent.id);
  if (!newParent) {
    return [newTree, NewFError("removeNodeFromRequestTree.noOldParent", "Error during remove", "Could not remove the selected request or group correctly", "no new parent found in remove")]
  }
  newParent.children = newParent.children.filter((current: RequestTreeNode) => current.id !== node.id);
  return [newTree, undefined];
}

export function newRequestTreeNode(): RequestTreeNode {
  return {
    id: newUUID(),
    name: "New Request",
    children: [],
    request: null,
    filepath: "",
    is_file_group: false
  };
}


export function newPrimeNode(): PrimeNode {
  return {
    id: newUUID(),
    key: 'TODO-KEY',
    label: 'TODO-LABEL',
    children: [],
    requestNode: undefined,
    groupNode: undefined,
    isFolder: false
  }
}


/**
 *
 * @param staleTree
 * @param dragNode
 * @param newDropNode - result from backend with the new node containing already the dragged node
 */
export function applyDragAndDropResult(staleTree: RequestTree, dragNode: RequestTreeNode, ddResult: DragAndDropResult): [RequestTree, FError] {

  let newDropNode = ddResult.new_drop_node;

  let dragParentId = findParentById(staleTree, dragNode.id)?.id;

  let newTree;

  if (staleTree.root.id == newDropNode.id) {
    newTree = { root: newDropNode };
  } else {
    newTree = cloneTree(staleTree);
    let copyDropNodeParent = findParentById(newTree, newDropNode.id);
    if (!copyDropNodeParent) {
      return [newTree, NewFError("dragAndDropResult.noCopyDropNodeParent", "Error during drag and drop", "Could not drag and drop element", "no copy drop node found")];
    }
    let drop_node_index = copyDropNodeParent.children.findIndex((child: RequestTreeNode) => {
      return child.id == newDropNode.id;
    });
    copyDropNodeParent.children[drop_node_index] = newDropNode;
  }

  let copyDragParent = findNode(newTree, dragParentId ?? '');
  if (!copyDragParent) {
    return [newTree, NewFError("dragAndDropResult.noCopyDragParent", "Error during drag and drop", "Could not drag and drop element", "no copy drag parent node found")];
  }
  // remove dragnode from parent
  copyDragParent.children = copyDragParent.children.filter((child: RequestTreeNode) => child.id !== dragNode.id);

  // if we remove a request from a file group that is now empty we need to remove the file group (parent of dragNode)
  // from its parent instead of remove the dragNode as a child from its old parent
  if (ddResult.remove_drag_node_parent) {
    let copyDragGrandParent = findParentById(newTree, dragNode.id);
    if (copyDragGrandParent) {
      copyDragGrandParent.children = copyDragGrandParent.children.filter((child: RequestTreeNode) => child.id !== copyDragParent?.id);
    }
  }

  return [newTree, undefined];
}

export function reorderReplace(requestTree: RequestTree, newNode: RequestTreeNode): [RequestTree, FError] {
  let newTree = cloneTree(requestTree);

  // we want to replace the root node, do not need to switch reference on parent
  if (newTree?.root?.id == newNode.id) {
    newTree.root = newNode;
    // no error, OK
    return [newTree, undefined];
  }

  let toReplaceParent = findParent(newTree, newNode);
  if (!toReplaceParent) {
    console.error("No replace parent found");
    return [newTree, NewFError("reorderNodeWithinParentReplace.noReplaceParent", "Could not reorder elements", "", "no replace parent found in reorderNodeWithinParentReplace")];
  }

  // replace node
  toReplaceParent.children = toReplaceParent.children.map((node: RequestTreeNode) => {
    if (node.id === newNode.id) {
      return newNode;
    } else {
      return node;
    }
  })

  return [newTree, undefined];
}


export function getAllRequestsFromTree(requestTree: RequestTree): RequestModel[] {
  let nodes = [requestTree.root]
  let requests: RequestModel[] = []

  while (nodes.length > 0) {
    let current = nodes.shift()
    if (!current) {
      continue
    }
    if (current.request) {
      requests.push(current.request)
    }

    if (current.children.length > 0) {
      nodes = [...nodes, ...current.children]
    }
  }

  return requests
}

export function getRequestsInSameGroup(requestModelId: UUID, requestTree: RequestTree): RequestModel[] {
  let nodes = [requestTree.root]

  while (nodes.length > 0) {
    let current = nodes.pop()
    if (!current) {
      continue
    }
    let childrenIds = current.children.map((child: RequestTreeNode) => child.request?.id).filter((id: string | undefined) => id !== undefined);
    if (childrenIds.includes(requestModelId)) {
      let requests: RequestModel[] = current.children.filter((node: RequestTreeNode) => {
        if (!node.request) {
          return false
        }
        return node.request.id !== requestModelId
      }).map((requestTreeNode: RequestTreeNode) => requestTreeNode.request) as RequestModel[]
      return requests
    }
    if (current.children.length > 0) {
      nodes = [...nodes, ...current.children];
    }
  }

  return [];
}

export function isModelSomewhereWithinGroup(modelId: UUID, groupNode: RequestTreeNode, requestTree: RequestTree): boolean {
  let nodes = [groupNode];
  while (nodes.length > 0) {
    let current = nodes.pop();
    if (!current) {
      continue;
    }
    let childrenIds = current.children.map((child: RequestTreeNode) => child.request?.id).filter((id: string | undefined) => id !== undefined);
    if (childrenIds.includes(modelId)) {
      return true;
    }
    if (current.children.length > 0) {
      nodes = [...nodes, ...current.children.filter((child: RequestTreeNode) => child.children.length > 0)]
    }
  }
  return false
}
