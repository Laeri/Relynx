import { Card } from "primereact/card";
import { Button } from "primereact/button";
import { useContext, useEffect, useState } from "react";
import { openCreateCollectionModal, openAddExistingCollectionsModal, openImportCollectionModal } from "../common/modal";
import { ToastContext } from "../App";
import { RelynxState, useRequestModelStore } from "../stores/requestStore";
import { AddCollectionsResult, LicenseData } from "../bindings";
import { TrialModal } from "../components/modals/TrialModal";
import { backend } from "../rpc";
import { catchError } from "../common/errorhandling";
import { checkLicenseDataValid } from "../common/license";
import { Message } from "primereact/message";


export interface ComponentProps {
}

export function OverviewComponent(props: ComponentProps) {

  const cardStyle = {
    marginTop: '20px'
  }

  const toast = useContext(ToastContext);

  const workspace = useRequestModelStore((state: RelynxState) => state.workspace);

  // if the trial modal has been shown already this session
  // within the trial period a user can click away the modal and it should not be shown every time they open the
  // overview again
  const trialShown = useRequestModelStore((state: RelynxState) => state.trialShown);
  const setTrialShown = useRequestModelStore((state: RelynxState) => state.setTrialShown);
  // after a few days the trial is not valid anymore, in this case entering license key is forced 
  const [isTrialValid, setIsTrialValid] = useState<boolean>(true);

  // license data which is saved in data directory and is checked every time the application is opened
  const [licenseData, setLicenseData] = useState<LicenseData | undefined>(undefined);

  // if true then the trial modal is opened
  const [showTrial, setShowTrial] = useState<boolean>(false);

  // if just activated we show a thank you message
  const [justActivated, setJustActivated] = useState<boolean>(false);

  const updateLicenseData = (newLicenseData: LicenseData) => {
    return backend.saveLicenseData(newLicenseData)
  }

  // Check license
  useEffect(() => {
    backend.loadLicenseData().then((licenseData: LicenseData) => {
      checkLicenseDataValid(licenseData).then((isValid: boolean) => {
        if (isValid) {
          return
        }

        if (trialShown) {
          return
        }

        if (licenseData.license_start === undefined || licenseData.license_start === null) {
          // start trial by setting the time and save the data
          licenseData = structuredClone(licenseData);
          licenseData.license_start = new Date().toISOString();
          updateLicenseData(licenseData).then(() => {
            // ignore
          }).catch(catchError);

        } else {
          let trialStartDate = Date.parse(licenseData.license_start);
          let now = Date.now();
          const diffTime = Math.abs(now - trialStartDate);
          const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));
          let trialDays = 3;
          let isTrialValid = diffDays < trialDays;
          setIsTrialValid(isTrialValid);
        }

        setTrialShown();
        setShowTrial(true);

        // const modalPromise = create(({ onResolve, onReject, isOpen }) => {
        //   return
        // });

        // modalPromise().then(() => {
        // });

      });

    });

  }, []);

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

      <TrialModal setActivated={() => setJustActivated(true)} updateLicenseData={updateLicenseData} licenseData={licenseData} isTrialValid={isTrialValid} isOpen={showTrial} onResolve={() => setShowTrial(false)} onReject={() => { }} />

      {
        justActivated && <Message style={{marginTop: '40px'}} severity="success" text="Your license has been activated successfully! Thank you for using Relynx! If you have any suggestions or feedback in general please write an email to info@relynx.app." />
      }

      <div style={{ marginTop: '50px', display: 'flex', justifyContent: 'space-between', width: '100%' }}>
        <Card title={<><i className={"pi pi-pencil mr-2"} />New Collection</>}
          subTitle={"Create a new collection in an empty folder"} style={cardStyle}
          className={"main-card"}
          footer={<Button label="New Collection" icon="pi pi-plus" onClick={() => openCreateCollectionModal(workspace)}
            style={{ minWidth: 0 }} />}
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
