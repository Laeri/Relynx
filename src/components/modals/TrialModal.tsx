import { Dialog } from "primereact/dialog";
import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";
import { useState } from "react";
import { checkLicenseStringValid, splitLicense } from "../../common/license";
import { LicenseData } from "../../bindings";
import { catchError } from "../../common/errorhandling";
import { RELYNX_BUY_LINK } from "../../common/common";

interface ComponentProps {
  isOpen: boolean
  onResolve: (result?: {}) => void
  onReject: () => void,
  isTrialValid: boolean,
  licenseData?: LicenseData,
  updateLicenseData: (newLicenseData: LicenseData) => Promise<null>,
  setActivated: () => void
}

export function TrialModal(props: ComponentProps) {

  const [license, setLicense] = useState<string | undefined>(undefined);

  const [isValid, setIsValid] = useState<boolean>(false);
  const [licenseError, setLicenseError] = useState<string>("");

  const licenseErrorMsg = "The license provided is not valid.\nCheck if you have entered it correctly";

  const updateLicense = (newLicense: string) => {
    setLicense(newLicense);
    checkLicenseStringValid(newLicense).then((isValid: boolean) => {
      if (isValid) {
        let license = splitLicense(newLicense);
        if (!license) {
          setIsValid(false);
          setLicenseError(licenseErrorMsg);
          return
        }
        setLicenseError("");
        setIsValid(true);
      } else {
        setLicenseError(licenseErrorMsg);
        setIsValid(false);
      }
    }).catch((_) => {
       setLicenseError(licenseErrorMsg);
      setIsValid(false);
    });
  }

  const closeTrialModal = () => {
    if (!license) {
      return
    }
    let newLicenseData = splitLicense(license);
    if (!newLicenseData) {
      return
    }

    props.setActivated();
    props.updateLicenseData(newLicenseData).then(() => {
      props.onResolve({ license });
    }).catch(catchError);
  }

  return (
    <Dialog header="Relynx Trial" visible={props.isOpen} dismissableMask={false}
      onHide={props.onResolve}
      closable={false}
      modal={true}
      footer={
        <div>
          {(props.isTrialValid && !isValid) &&
            <Button disabled={!props.isTrialValid} label="Continue Trial" icon="pi pi-times" className={'p-button-secondary p-button-text'}
              onClick={() => props.onResolve()} />
          }
          <Button disabled={!isValid} label="Activate License" icon="pi pi-check"
            onClick={() => closeTrialModal()}
            style={{ marginLeft: '80px' }} />
        </div>
      }>
      <div style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'flex-start',
        marginBottom: '0px',
        marginTop: '20px'
      }}>

        <div style={{ display: 'flex', flexDirection: 'column', width: '100%', marginBottom: '20px', alignItems: 'flex-start' }}>

          <h3 style={{}}>Relynx Trial</h3>

          {
            props.isTrialValid &&
            <>
              <p style={{ marginTop: '20px' }}>You can test out Relynx for 3 days in the trial version with all features enabled.</p>
              <p style={{ marginTop: '20px' }}>Afterwards you can obtain a license here: <a href={RELYNX_BUY_LINK} target="_blank">{RELYNX_BUY_LINK}</a></p>
            </>
          }
          {
            !props.isTrialValid &&
            <>
              <p style={{ marginTop: '20px' }}>You have reached the end of the trial version. Thanks for trying out Relynx.</p>
              <p style={{ marginTop: '20px' }}>A valid license can be obtained here: <a href={RELYNX_BUY_LINK} target="_blank">{RELYNX_BUY_LINK}</a></p>
            </>
          }
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', marginTop: '20px' }}>
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
              <InputText style={{ marginTop: '20px', width: '300px' }} placeholder="License Key" value={license ?? ""} onChange={(event: any) => updateLicense(event.target.value)} />
              {
                !isValid &&
                <small style={{ marginTop: '10px', height: '20px' }} className="text-danger">{licenseError}</small>
              }
              {
                isValid &&
                <small style={{ marginTop: '10px', height: '20px', color: '#9fdaa8' }}>The license is valid and can be activated.</small>
              }
            </div>
          </div>
        </div>
      </div>
    </Dialog >
  )
}
