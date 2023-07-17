import { LicenseData } from "../bindings";
import { backend } from "../rpc";

export const checkLicenseStringValid = (license: string): Promise<boolean> => {
  return new Promise<boolean>((resolve: any, reject: any) => {
    let licenseData = splitLicense(license);
    if (licenseData === undefined || licenseData === null) {
      resolve(false);
      return
    }
    checkLicenseDataValid(licenseData).then((result: boolean) => resolve(result)).catch((err) => reject(err));
  });
};

export const checkLicenseDataValid = (licenseData: LicenseData): Promise<boolean> => {
  return new Promise<boolean>((resolve: any, reject: any) => {
    if (licenseData === undefined) {
      resolve(false);
      return;
    }
    if (licenseData.license_key === undefined || licenseData.license_key === null) {
      resolve(false);
      return;
    }
    if (licenseData.license_signature === undefined || licenseData.license_signature === null) {
      resolve(false);
      return;
    }
    backend.isSignatureValid(licenseData).then((result: boolean) => resolve(result)).catch((err) => reject(err));
  });
};


export const splitLicense = (license: string): LicenseData | undefined => {
  let split = license.split(":");
  if (split.length != 2) {
    return undefined;
  }

  return {
    license_key: split[0],
    license_signature: split[1],
    license_start: null
  }
}
