import { Dialog } from "primereact/dialog";
import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";
import { useContext, useState } from "react";
import { onCtrlEnter } from "../../common/eventhandling";
import { findNode, PrimeNode, renameGroupNode } from "../../common/treeUtils";
import { backend } from "../../rpc";
import { Collection, ValidateGroupNameResult } from "../../bindings";
import { catchError } from "../../common/errorhandling";
import { RelynxState, useRequestModelStore } from "../../stores/requestStore";
import { ToastContext } from "../../App";


interface ComponentProps {
  isOpen: boolean
  onResolve: (groupName?: string) => void
  onReject: () => void,
  node: PrimeNode,
  collection: Collection
}

export function EditGroupNameModal(props: ComponentProps) {

  const [newGroupName, setNewGroupName] = useState<string>(props.node.groupNode?.name ?? 'New Group');
  const [sanitizedName, setSanitizedName] = useState<string | undefined>();
  const [pathExists, setPathExists] = useState<boolean>(false);
  const [newPath, setNewPath] = useState<string | undefined>();
  const toast = useContext(ToastContext);


  const requestTree = useRequestModelStore((state: RelynxState) => state.requestTree);
  const updateRequestTree = useRequestModelStore((state: RelynxState) => state.updateRequestTree);

  const renameGroup = () => {
    if (newPath === undefined) {
      return;
    }
    backend.renameGroup(props.collection.path, props.node.groupNode?.filepath as string, newGroupName)
      .then((newPath: string) => {
        if (!requestTree) {
          return
        }
        let groupNode = findNode(requestTree, props.node.key);
        if (!groupNode) {
          return
        }
        let [newTree, error] = renameGroupNode(requestTree, props.node.key, { newName: newGroupName, newPath: newPath });
        if (error) {
          console.error(error);
          toast.showError(error.message ?? "Could not update the ui with the new group node. Try to reopen the application.", "");
          return;
        }
        updateRequestTree(newTree);
        props.onResolve(newGroupName);
      }).catch(catchError);
  }

  const validateGroupName = () => {
    backend.validateGroupName(props.node.groupNode?.filepath ?? '', newGroupName).then((result: ValidateGroupNameResult) => {
      if (result.sanitized_name !== newGroupName) {
        setSanitizedName(result.sanitized_name);
      } else {
        setSanitizedName(undefined);
      }

      setPathExists(result.path_exists_already);

      // if the path exists already or the current input is not yet sanitized we have no new path to use for creating the group...
      if (result.path_exists_already || result.sanitized_name !== newGroupName) {
        setNewPath(undefined);
      } else {
        setNewPath(result.new_path);
      }

    }).catch(catchError);

  }

  return (
    <Dialog header="Rename Group" visible={props.isOpen} dismissableMask={false}
      style={{ width: '50vw' }}
      onHide={props.onResolve}
      footer={
        <div>
          <Button label="Cancel" icon="pi pi-times" className={'p-button-secondary p-button-text'}
            onClick={() => props.onResolve()} />
          <Button disabled={newPath === undefined} label="Rename" icon="pi pi-check"
            onClick={renameGroup}
            style={{ marginLeft: '80px' }} />
        </div>
      }>
      <div style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'flex-start',
        marginBottom: '50px',
        marginTop: '40px',
        flexGrow: 1
      }}>
        <div style={{ display: 'flex', width: '100%', marginBottom: '0px', alignItems: 'center' }}>
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'start' }}>
            <div style={{ display: 'flex', alignItems: 'center' }}>
              <h3>Name</h3>
              <InputText autoFocus={true} onKeyUp={validateGroupName} onKeyPress={(event: any) => onCtrlEnter(event, renameGroup)} value={newGroupName} onChange={(e) => setNewGroupName(e.target.value)}
                style={{ marginLeft: '20px', flexBasis: '60%' }} />
            </div>

            <div style={{ marginTop: '10px', minHeight: '50px' }}>
              {
                (sanitizedName !== undefined && sanitizedName !== newGroupName) &&
                <span className='p-error'>A valid folder name is required. For example use '{sanitizedName}'. Each group gets its own folder on the file system.</span>
              }

            </div>

            <div style={{ marginTop: '10px', minHeight: '30px' }}>
              {
                pathExists &&
                <span className='p-error'>There exists already a group with the same name. Choose another name.</span>
              }

            </div>
          </div>
        </div>
      </div>

    </Dialog >
  )
}
