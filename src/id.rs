use bson::oid::ObjectId;
use cuid;
use nanoid::nanoid;
use std::str::FromStr;
use ulid;
use uuid::Uuid;

#[derive(Debug)]
pub enum IDError {
    MissingNamespace(String),
    MissingName(String),
    InvalidNamespace(String),
    // There are several potential CuidError states but all of them
    // seem to be caused by OS errors so I've just shimmed this for now
    CuidError(cuid::CuidError),
}

impl std::fmt::Display for IDError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IDError::MissingNamespace(msg) => write!(f, "{}", msg),
            IDError::MissingName(msg) => write!(f, "{}", msg),
            IDError::InvalidNamespace(msg) => write!(f, "{}", msg),
            IDError::CuidError(err) => write!(f, "{}", err.to_string()), // This isn't great but should be fine
        }
    }
}

impl std::error::Error for IDError {}

/// Enum representing different ID formats and versions
#[derive(Debug, Clone)]
pub enum IDFormat {
    Simple(UuidVersion),
    Hyphenated(UuidVersion),
    URN(UuidVersion),
    OID,
    NanoID,
    Ulid,
    Cuid(CuidVersion),
}

/// Internal enum for UUID versions
#[derive(Debug, Clone, Copy)]
pub enum UuidVersion {
    V1,
    V3,
    V4,
    V5,
}

/// Internal enum for CUID versions
#[derive(Debug, Clone, Copy)]
pub enum CuidVersion {
    V1,
    V2,
}

/**
 * Returns the newly generated id
 *
 * # Arguments
 *
 * * `id_format` - The format of the ID to generate
 * * `len` - The length of the ID (only applicable for NanoID)
 * * `namespace` - The namespace for UUID v3 and v5 (required for those versions)
 * * `name` - The name for UUID v3 and v5 (required for those versions)
 *
 * # Returns
 *
 * A string representing the generated ID
 */
pub fn new_id(
    id_format: &IDFormat,
    len: Option<usize>,
    namespace: Option<&str>,
    name: Option<&str>,
) -> Result<String, IDError> {
    match id_format {
        IDFormat::Simple(version) => Ok(generate_uuid(*version, namespace, name)?
            .simple()
            .to_string()),
        IDFormat::Hyphenated(version) => Ok(generate_uuid(*version, namespace, name)?
            .hyphenated()
            .to_string()),
        IDFormat::URN(version) => Ok(generate_uuid(*version, namespace, name)?.urn().to_string()),
        IDFormat::OID => Ok(ObjectId::new().to_string()),
        IDFormat::NanoID => {
            let l = len.unwrap_or(21);
            Ok(nanoid!(l))
        }
        IDFormat::Cuid(version) => Ok(generate_cuid(*version))?,
        IDFormat::Ulid => Ok(ulid::Ulid::new().to_string()),
    }
}

fn generate_uuid(
    version: UuidVersion,
    namespace: Option<&str>,
    name: Option<&str>,
) -> Result<Uuid, IDError> {
    match version {
        UuidVersion::V1 => Ok(Uuid::now_v1(&[1, 2, 3, 4, 5, 6])),
        UuidVersion::V3 => {
            let namespace = namespace.ok_or_else(||
                IDError::MissingNamespace("UUID v3 requires --namespace parameter. Example: --namespace 6ba7b810-9dad-11d1-80b4-00c04fd430c8".to_string())
            )?;
            let name = name.ok_or_else(|| {
                IDError::MissingName(
                    "UUID v3 requires --name parameter. Example: --name example.com".to_string(),
                )
            })?;
            let namespace = Uuid::from_str(namespace).map_err(|_|
                IDError::InvalidNamespace("Invalid namespace UUID format. Must be a valid UUID like 6ba7b810-9dad-11d1-80b4-00c04fd430c8.\nCommon namespaces:\n  - DNS: 6ba7b810-9dad-11d1-80b4-00c04fd430c8\n  - URL: 6ba7b811-9dad-11d1-80b4-00c04fd430c8".to_string())
            )?;
            Ok(Uuid::new_v3(&namespace, name.as_bytes()))
        }
        UuidVersion::V4 => Ok(Uuid::new_v4()),
        UuidVersion::V5 => {
            let namespace = namespace.ok_or_else(||
                IDError::MissingNamespace("UUID v5 requires --namespace parameter. Example: --namespace 6ba7b810-9dad-11d1-80b4-00c04fd430c8".to_string())
            )?;
            let name = name.ok_or_else(|| {
                IDError::MissingName(
                    "UUID v5 requires --name parameter. Example: --name example.com".to_string(),
                )
            })?;
            let namespace = Uuid::from_str(namespace).map_err(|_|
                IDError::InvalidNamespace("Invalid namespace UUID format. Must be a valid UUID like 6ba7b810-9dad-11d1-80b4-00c04fd430c8.\nCommon namespaces:\n  - DNS: 6ba7b810-9dad-11d1-80b4-00c04fd430c8\n  - URL: 6ba7b811-9dad-11d1-80b4-00c04fd430c8".to_string())
            )?;
            Ok(Uuid::new_v5(&namespace, name.as_bytes()))
        }
    }
}

fn generate_cuid(version: CuidVersion) -> Result<String, IDError> {
    match version {
        CuidVersion::V1 => cuid::cuid1().map_err(|err| IDError::CuidError(err)),
        CuidVersion::V2 => Ok(cuid::cuid2()),
    }
}
