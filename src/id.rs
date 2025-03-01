use nanoid::nanoid;
use bson::oid::ObjectId;
use uuid::Uuid;

#[derive(Debug)]
pub enum IDFormat {
    Simple,
    Hyphenated,
    URN,
    OID,
    NanoID,
}

/**
 * Returns the newly generated id
 */
pub fn new_id(id_format: &IDFormat, len: Option<usize>) -> String {
    match id_format {
        IDFormat::Simple => Uuid::new_v4().simple().to_string(),
        IDFormat::Hyphenated => Uuid::new_v4().to_string(),
        IDFormat::URN => Uuid::new_v4().urn().to_string(),
        IDFormat::OID => ObjectId::new().to_string(),
        IDFormat::NanoID => {
            let l = len.unwrap();
            nanoid!(l)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_uuid() {
        let id = new_id(&IDFormat::Simple, None);
        assert_eq!(id.len(), 32);
        assert!(!id.contains('-'));
    }

    #[test]
    fn test_hyphenated_uuid() {
        let id = new_id(&IDFormat::Hyphenated, None);
        assert_eq!(id.len(), 36);
        assert_eq!(id.matches('-').count(), 4);
    }

    #[test]
    fn test_urn_uuid() {
        let id = new_id(&IDFormat::URN, None);
        assert!(id.starts_with("urn:uuid:"));
        assert_eq!(id.len(), 45);
    }

    #[test]
    fn test_objectid() {
        let id = new_id(&IDFormat::OID, None);
        assert_eq!(id.len(), 24);
        // ObjectID should be hex
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_nanoid() {
        let len = 10;
        let id = new_id(&IDFormat::NanoID, Some(len));
        assert_eq!(id.len(), len);
    }

    #[test]
    #[should_panic]
    fn test_nanoid_requires_length() {
        new_id(&IDFormat::NanoID, None);
    }
}
