use crate::{
    config::get_data_dir,
    error::RelynxError,
    get_license_pub_key,
};

use base64::{
    engine::{self, general_purpose},
    Engine,
};
use rsa::{
    pkcs1::DecodeRsaPublicKey,
    pkcs1v15::{Signature, VerifyingKey},
    signature::Verifier,
    RsaPublicKey,
};

use rsa::sha2::Sha256;
use rspc::Type;
use serde::{Deserialize, Serialize};

const LICENSE_FILENAME: &str = "license.json";

#[derive(Serialize, Deserialize)]
pub struct License {
    email: String,
    id: u64,
}

type ISO8601 = String;
// is base64 encoded string
type Base64EncodedString = String;
// is hex encoded bytes
type HexEncodedString = String;
#[derive(Serialize, Deserialize, Type, Default, Debug)]
pub struct LicenseData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_key: Option<Base64EncodedString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_signature: Option<HexEncodedString>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_start: Option<ISO8601>,
}

pub fn get_license_data() -> Result<LicenseData, RelynxError> {
    let data_dir = get_data_dir().ok_or(RelynxError::LoadLicenseDataError)?;
    let license_path = data_dir.join(LICENSE_FILENAME);

    let license_data = if !license_path.exists() {
        let license_data = LicenseData {
            license_key: None,
            license_signature: None,
            license_start: None,
        };
        let _ = save_license_data(&license_data);
        license_data
    } else {
        let content = std::fs::read_to_string(&license_path).map_err(|io_err| {
            log::error!("Could not read license to string, path: '{}", license_path.display());
            log::error!("Io Error: {:?}", io_err);
            RelynxError::LoadLicenseDataError
        })?;
        let license_data: LicenseData = serde_json::from_str(&content).map_err(|err| {
            log::error!(
                "Could not deserialize content to license data, content: '{:?}'",
                content
            );
            log::error!("Serde Error: {:?}", err);
            RelynxError::LoadLicenseDataError
        })?;
        license_data
    };

    Ok(license_data)
}

pub fn save_license_data(license_data: &LicenseData) -> Result<(), RelynxError> {
    let data_dir = get_data_dir().ok_or(RelynxError::SaveLicenseDataError)?;
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir).map_err(|err| {
            log::error!(
                "During save license data cannot create data directory: '{}'",
                data_dir.display()
            );
            RelynxError::SaveLicenseDataError
        })?;
    }

    let content = serde_json::to_string_pretty::<LicenseData>(license_data).map_err(|err| {
        log::error!("Could not serialize license data: '{:?}'", license_data);
        log::error!("Serde Error: {:?}", err);
        RelynxError::SaveLicenseDataError
    })?;

    let license_path = data_dir.join(LICENSE_FILENAME);
    std::fs::write(&license_path, content)
        .map(|_| ())
        .map_err(|err| {
            log::error!(
                "Could not write license data to file, path: '{}'",
                license_path.display()
            );
            RelynxError::SaveLicenseDataError
        })
}

pub fn verify_signature(license_data: &LicenseData) -> Result<bool, RelynxError> {
    if license_data.license_signature.is_none() || license_data.license_key.is_none() {
        log::error!(
            "Missing license signature or key, license_data: {:?}",
            license_data
        );
        return Ok(false);
    }

    let license_value = license_data.license_key.as_ref().unwrap();
    const CUSTOM_ENGINE: engine::GeneralPurpose =
        engine::GeneralPurpose::new(&base64::alphabet::STANDARD, general_purpose::PAD);

    let base64_decoded_payload = Engine::decode(&CUSTOM_ENGINE, license_value).map_err(|err| {
        log::error!("License Data: {:?}", license_data);
        log::error!("Could not calculate license signature. License value is not valid base64.");
        log::error!("Base64 decode err: {:?}", err);
        log::error!("License Data: {:?}", license_data);

        RelynxError::LicenseInvalid
    })?;

    let license_signature = Engine::decode(
        &CUSTOM_ENGINE,
        license_data.license_signature.as_ref().unwrap(),
    )
    .map_err(|err| {
        log::error!(
            "Could not calculate license signature. License signature is not valid base64."
        );
        log::error!("Base64 decode err: {:?}", err);
        log::error!("License Data: {:?}", license_data);

        RelynxError::LicenseInvalid
    })?;

    let pub_key = get_license_pub_key().map_err(|err| {
        log::error!("Could not access pub key: {:?}", err);
        RelynxError::LicenseInvalid
    })?;

    let pub_key = RsaPublicKey::from_pkcs1_pem(&pub_key).unwrap();

    let signature = Signature::try_from(license_signature.as_slice()).map_err(|err| {
        log::error!("Signature error: {:?}", err);
        log::error!("Could not calculate license signature. Signature is not in valid format.");
        log::error!("License Signature: {:?}", license_signature);
        log::error!("License Data: {:?}", license_data);

        RelynxError::LicenseInvalid
    })?;

    let verifying_key_openssl: VerifyingKey<Sha256> = VerifyingKey::new(pub_key.clone());
    let result = verifying_key_openssl.verify(base64_decoded_payload.as_slice(), &signature);
    if result.is_err() {
        return Ok(false);
    }

    let decoded_payload_str =
        std::str::from_utf8(base64_decoded_payload.as_slice()).map_err(|_err| {
            log::error!(
                "License signature can not be decoded. Payload cannot be converted into a string."
            );
            log::error!("Base64 Payload: {:?}", base64_decoded_payload);
            log::error!("License Data: {:?}", license_data);
            RelynxError::LicenseInvalid
        })?;

    let _ = serde_json::from_str::<License>(decoded_payload_str).map_err(|_err| {
        log::error!("License signature can not be decoded. Cannot convert payload to json.");
        log::error!("Decoded Payload: {:?}", decoded_payload_str);
        log::error!("License Data: {:?}", license_data);

        RelynxError::LicenseInvalid
    })?;

    // if we got to here we could decode correctly the license and the signature matches
    return Ok(true);
}
