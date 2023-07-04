import { Dialog } from "primereact/dialog";
import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";
import { useState } from "react";
import { checkLicenseStringValid, splitLicense } from "../../common/license";
import { LicenseData } from "../../bindings";
import { catchError } from "../../common/errorhandling";
import { RELYNX_WEBSITE } from "../../common/common";

interface ComponentProps {
  isOpen: boolean
  onResolve: (result?: {}) => void
  onReject: () => void,
  isTrialValid: boolean,
  licenseData?: LicenseData,
  updateLicenseData: (newLicenseData: LicenseData) => Promise<null>
}

export function TrialModal(props: ComponentProps) {

  const [license, setLicense] = useState<string | undefined>(undefined);

  const [isValid, setIsValid] = useState<boolean>(false);
  const [licenseError, setLicenseError] = useState<string>("");

  const licenseErrorMsg = "The license provided is not valid. Check if you have entered it correctly";

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
        setIsValid(true);;
        let newLicenseData: LicenseData = {
          license_key: undefined,
          license_signature: undefined,
          license_start: undefined
        };
        if (props.licenseData) {
          newLicenseData = structuredClone(props.licenseData);
        }
        newLicenseData.license_key = license?.license_key;
        newLicenseData.license_signature = license?.license_signature;
        props.updateLicenseData(newLicenseData).then(() => {
          props.onResolve({ license })
        }).catch(catchError);
      } else {
        setLicenseError(licenseErrorMsg);
        setIsValid(false);
      }
    }).catch((_) => {
      setLicenseError(licenseErrorMsg);
      setIsValid(false);
    });
  }

  return (
    <Dialog header="Relynx Trial" visible={props.isOpen} dismissableMask={false}
      onHide={props.onResolve}
      closable={false}
      modal={true}
      footer={
        <div>
          {props.isTrialValid &&
            <Button disabled={!props.isTrialValid} label="Continue Trial" icon="pi pi-times" className={'p-button-secondary p-button-text'}
              onClick={() => props.onResolve()} />
          }
          <Button disabled={!isValid} label="Activate License" icon="pi pi-check"
            onClick={() => props.onResolve({})}
            style={{ marginLeft: '80px' }} />
        </div>
      }>
      <div style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'flex-start',
        marginBottom: '0px',
        marginTop: '40px'
      }}>

        <div style={{ display: 'flex', flexDirection: 'column', width: '100%', marginBottom: '20px', alignItems: 'flex-start' }}>

          <h3 style={{}}>Relynx Trial</h3>

          <p style={{ marginTop: '20px' }}>You can test out relynx for 3 days in the trial version with all features enabled.</p>

          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', marginTop: '20px' }}>
            <p>Afterwards open <a href={RELYNX_WEBSITE} target="_blank">relynx.app</a> in your browser to obtain a license and enter it below.</p>
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
              <InputText style={{ marginTop: '20px', width: '300px' }} placeholder="License Key" value={license ?? ""} onChange={(event: any) => updateLicense(event.target.value)} />
              <small style={{ marginTop: '10px', height: '20px' }} className="text-danger">{licenseError}</small>
            </div>
          </div>
        </div>
      </div>
    </Dialog >
  )
}
