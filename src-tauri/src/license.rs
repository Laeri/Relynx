use std::io::Read;

use crate::{
    config::get_data_dir,
    error::{DisplayErrorKind, FrontendError},
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

pub fn get_license_data() -> Result<LicenseData, FrontendError> {
    let error =
        FrontendError::new_with_message(DisplayErrorKind::Generic, "Could not load license data");
    let data_dir = get_data_dir().ok_or(error.clone())?;
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
        let content = std::fs::read_to_string(&license_path).map_err(|_io_err| error.clone())?;
        let license_data: LicenseData =
            serde_json::from_str(&content).map_err(|_err| error.clone())?;
        license_data
    };

    Ok(license_data)
}

pub fn save_license_data(license_data: &LicenseData) -> Result<(), FrontendError> {
    let error =
        FrontendError::new_with_message(DisplayErrorKind::Generic, "Could not load license data");

    let data_dir = get_data_dir().ok_or(error.clone())?;
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir).map_err(|_err| error.clone())?;
    }

    let content =
        serde_json::to_string_pretty::<LicenseData>(license_data).map_err(|_err| error.clone())?;

    let license_path = data_dir.join(LICENSE_FILENAME);
    std::fs::write(&license_path, content)
        .map(|_| ())
        .map_err(|_err| error.clone())
}

pub fn verify_signature(license_data: &LicenseData) -> Result<bool, FrontendError> {
    if license_data.license_signature.is_none() || license_data.license_key.is_none() {
        return Ok(false);
    }

    let license_value = license_data.license_key.as_ref().unwrap();
    const CUSTOM_ENGINE: engine::GeneralPurpose =
        engine::GeneralPurpose::new(&base64::alphabet::STANDARD, general_purpose::PAD);

    let base64_decoded_payload = Engine::decode(&CUSTOM_ENGINE, license_value).map_err(|_err| {
        FrontendError::new_with_message(
            DisplayErrorKind::Generic,
            "Could not calculate license signature. License value is not valid base64.",
        )
    })?;

    let license_signature = Engine::decode(
        &CUSTOM_ENGINE,
        license_data.license_signature.as_ref().unwrap(),
    )
    .map_err(|_err| {
        FrontendError::new_with_message(
            DisplayErrorKind::Generic,
            "Could not calculate license signature. License Signature is not valid base64.",
        )
    })?;

    let pub_key = get_license_pub_key().map_err(|_err| {
        FrontendError::new_with_message(
            DisplayErrorKind::Generic,
            "Could not calculate license signature. Public key is not present.",
        )
    })?;

    let pub_key = RsaPublicKey::from_pkcs1_pem(&pub_key).unwrap();

    let signature = Signature::try_from(license_signature.as_slice()).map_err(|_err| {
        FrontendError::new_with_message(
            DisplayErrorKind::Generic,
            "Could not calculate license signature. Signature is not in valid format.",
        )
    })?;

    let verifying_key_openssl: VerifyingKey<Sha256> = VerifyingKey::new(pub_key.clone());
    let result = verifying_key_openssl.verify(base64_decoded_payload.as_slice(), &signature);
    if result.is_err() {
        return Ok(false);
    }

    let decoded_payload_str =
        std::str::from_utf8(base64_decoded_payload.as_slice()).map_err(|_err| {
            FrontendError::new_with_message(
                DisplayErrorKind::Generic,
                "License signature can not be decoded. Payload cannot be converted into a string.",
            )
        })?;

    let _ = serde_json::from_str::<License>(decoded_payload_str).map_err(|_err| {
        FrontendError::new_with_message(
            DisplayErrorKind::Generic,
            "License signature can not be decoded. Cannot convert payload to json.",
        )
    })?;

    // if we got to here we could decode correctly the license and the signature matches
    return Ok(true);
}
