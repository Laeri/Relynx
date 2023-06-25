import { Card } from "primereact/card";
import { Button } from "primereact/button";
import { useContext } from "react";
import { openCreateCollectionModal, openAddExistingCollectionsModal, openImportCollectionModal } from "../common/modal";
import { ToastContext } from "../App";
import { useRequestModelStore } from "../stores/requestStore";
import { AddCollectionsResult, Collection } from "../bindings";



export interface ComponentProps {

}

export function OverviewComponent(_props: ComponentProps) {

  const cardStyle = {
    marginTop: '20px'
  }

  const toast = useContext(ToastContext);

  const workspace = useRequestModelStore((state) => state.workspace);


  const doOpenAddExistingCollectionModal = () => {
    openAddExistingCollectionsModal(workspace, toast).then((_result: void | AddCollectionsResult) => {
      // @TODO: what about errored collections?
    });
  }


  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'flex-start',
      paddingTop: '50px',
      position: 'relative'
    }}>
      <h1>Relynx</h1>
      {
        workspace.collections.length == 0 && <p style={{ marginTop: '20px', textAlign: 'left' }}>
          You haven't created any collections yet in your workspace.
          Either create a new collection from scratch, import the folder of an existing relynx collection, or use the
          importer to convert an external collection into the relynx format.
        </p>
      }
      <div style={{ marginTop: '50px', display: 'flex', justifyContent: 'space-between', width: '100%' }}>
        <Card title={<><i className={"pi pi-pencil mr-2"} />New Collection</>}
          subTitle={"Create a new collection in an empty folder"} style={cardStyle}
          className={"main-card"}
          footer={<Button label="New Collection" icon="pi pi-plus" onClick={() => openCreateCollectionModal(workspace)}
            style={{minWidth: 0}} />}
        />
        <Card title={<><i className={"pi pi-plus mr-2"} />Add Existing</>}
          subTitle={"Add existing relynx collection from folder"} className={"main-card ml-8"} style={cardStyle}
          footer={<Button label="Add Collection" icon="pi pi-plus"
            onClick={doOpenAddExistingCollectionModal}
            style={{ marginTop: 'auto' }} />}
        />

        <Card title={<><i className={"pi pi-arrow-down-left mr-2"} />Import Collection</>}
          subTitle={"Import collection from external source"} className={"main-card ml-8"} style={cardStyle}
          footer={<Button label={"Import Collection"} icon="pi pi-plus"
            onClick={() => openImportCollectionModal(workspace)} />}
        />
      </div>
    </div>
  )
}
