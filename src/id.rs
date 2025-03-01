use nanoid::nanoid;
use bson::oid::ObjectId;
use uuid::Uuid;
use std::str::FromStr;

/// Enum representing different ID formats and versions
#[derive(Debug, Clone)]
pub enum IDFormat {
    Simple(UuidVersion),
    Hyphenated(UuidVersion),
    URN(UuidVersion),
    OID,
    NanoID,
}

/// Internal enum for UUID versions
#[derive(Debug, Clone, Copy)]
pub enum UuidVersion {
    V1,
    V3,
    V4,
    V5,
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
pub fn new_id(id_format: &IDFormat, len: Option<usize>, namespace: Option<&str>, name: Option<&str>) -> String {
    match id_format {
        IDFormat::Simple(version) => {
            let uuid = generate_uuid(*version, namespace, name);
            uuid.simple().to_string()
        },
        IDFormat::Hyphenated(version) => {
            let uuid = generate_uuid(*version, namespace, name);
            uuid.hyphenated().to_string()
        },
        IDFormat::URN(version) => {
            let uuid = generate_uuid(*version, namespace, name);
            uuid.urn().to_string()
        },
        IDFormat::OID => ObjectId::new().to_string(),
        IDFormat::NanoID => {
            let l = len.expect("Length must be provided for NanoID");
            nanoid!(l)
        }
    }
}

fn generate_uuid(version: UuidVersion, namespace: Option<&str>, name: Option<&str>) -> Uuid {
    match version {
        UuidVersion::V1 => {
            // For v1, we'll use the current timestamp
            Uuid::now_v1(&[1, 2, 3, 4, 5, 6])
        },
        UuidVersion::V3 => {
            let namespace = Uuid::from_str(namespace.expect("Namespace required for UUID v3")).expect("Invalid namespace UUID");
            let name = name.expect("Name required for UUID v3");
            Uuid::new_v3(&namespace, name.as_bytes())
        },
        UuidVersion::V4 => Uuid::new_v4(),
        UuidVersion::V5 => {
            let namespace = Uuid::from_str(namespace.expect("Namespace required for UUID v5")).expect("Invalid namespace UUID");
            let name = name.expect("Name required for UUID v5");
            Uuid::new_v5(&namespace, name.as_bytes())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_uuid_v4() {
        let id = new_id(&IDFormat::Simple(UuidVersion::V4), None, None, None);
        assert_eq!(id.len(), 32);
        assert!(!id.contains('-'));
    }

    #[test]
    fn test_hyphenated_uuid_v4() {
        let id = new_id(&IDFormat::Hyphenated(UuidVersion::V4), None, None, None);
        assert_eq!(id.len(), 36);
        assert_eq!(id.matches('-').count(), 4);
    }

    #[test]
    fn test_urn_uuid_v4() {
        let id = new_id(&IDFormat::URN(UuidVersion::V4), None, None, None);
        assert!(id.starts_with("urn:uuid:"));
        assert_eq!(id.len(), 45);
    }

    #[test]
    fn test_uuid_v3() {
        let namespace = "6ba7b810-9dad-11d1-80b4-00c04fd430c8"; // UUID namespace for URLs
        let name = "example.com";
        let id = new_id(&IDFormat::Simple(UuidVersion::V3), None, Some(namespace), Some(name));
        assert_eq!(id.len(), 32);
    }

    #[test]
    fn test_uuid_v5() {
        let namespace = "6ba7b810-9dad-11d1-80b4-00c04fd430c8"; // UUID namespace for URLs
        let name = "example.com";
        let id = new_id(&IDFormat::Simple(UuidVersion::V5), None, Some(namespace), Some(name));
        assert_eq!(id.len(), 32);
    }

    #[test]
    fn test_objectid() {
        let id = new_id(&IDFormat::OID, None, None, None);
        assert_eq!(id.len(), 24);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_nanoid() {
        let len = 10;
        let id = new_id(&IDFormat::NanoID, Some(len), None, None);
        assert_eq!(id.len(), len);
    }

    #[test]
    #[should_panic(expected = "Length must be provided for NanoID")]
    fn test_nanoid_requires_length() {
        new_id(&IDFormat::NanoID, None, None, None);
    }

    #[test]
    #[should_panic(expected = "Namespace required for UUID v3")]
    fn test_uuid_v3_requires_namespace() {
        new_id(&IDFormat::Simple(UuidVersion::V3), None, None, Some("test"));
    }

    #[test]
    #[should_panic(expected = "Name required for UUID v3")]
    fn test_uuid_v3_requires_name() {
        new_id(&IDFormat::Simple(UuidVersion::V3), None, Some("6ba7b810-9dad-11d1-80b4-00c04fd430c8"), None);
    }
}
