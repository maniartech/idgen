use idgen::id::{new_id, CuidVersion, IDError, IDFormat, UuidVersion};

// ============================================
// UUID v4 Tests
// ============================================

#[test]
fn test_simple_uuid_v4() {
    let id = new_id(&IDFormat::Simple(UuidVersion::V4), None, None, None).unwrap();
    assert_eq!(id.len(), 32);
    assert!(!id.contains('-'));
}

#[test]
fn test_hyphenated_uuid_v4() {
    let id = new_id(&IDFormat::Hyphenated(UuidVersion::V4), None, None, None).unwrap();
    assert_eq!(id.len(), 36);
    assert_eq!(id.matches('-').count(), 4);
}

#[test]
fn test_urn_uuid_v4() {
    let id = new_id(&IDFormat::URN(UuidVersion::V4), None, None, None).unwrap();
    assert!(id.starts_with("urn:uuid:"));
    assert_eq!(id.len(), 45);
}

#[test]
fn test_uuid_v3() {
    let namespace = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
    let name = "example.com";
    let id = new_id(
        &IDFormat::Simple(UuidVersion::V3),
        None,
        Some(namespace),
        Some(name),
    )
    .unwrap();
    assert_eq!(id.len(), 32);
}

#[test]
fn test_uuid_v5() {
    let namespace = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
    let name = "example.com";
    let id = new_id(
        &IDFormat::Simple(UuidVersion::V5),
        None,
        Some(namespace),
        Some(name),
    )
    .unwrap();
    assert_eq!(id.len(), 32);
}

#[test]
fn test_objectid() {
    let id = new_id(&IDFormat::OID, None, None, None).unwrap();
    assert_eq!(id.len(), 24);
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_nanoid() {
    let len = 10;
    let id = new_id(&IDFormat::NanoID, Some(len), None, None).unwrap();
    assert_eq!(id.len(), len);
}

#[test]
fn test_nanoid_default_length() {
    let id = new_id(&IDFormat::NanoID, None, None, None).unwrap();
    assert_eq!(id.len(), 21);
}

#[test]
fn test_uuid_v3_requires_namespace() {
    let result = new_id(&IDFormat::Simple(UuidVersion::V3), None, None, Some("test"));
    assert!(matches!(result, Err(IDError::MissingNamespace(_))));
}

#[test]
fn test_uuid_v3_requires_name() {
    let result = new_id(
        &IDFormat::Simple(UuidVersion::V3),
        None,
        Some("6ba7b810-9dad-11d1-80b4-00c04fd430c8"),
        None,
    );
    assert!(matches!(result, Err(IDError::MissingName(_))));
}

#[test]
fn test_cuid_v1() {
    let id = new_id(&IDFormat::Cuid(CuidVersion::V1), None, None, None).unwrap();
    assert!(cuid::is_cuid1(id));
}

#[test]
fn test_cuid_v2() {
    let id = new_id(&IDFormat::Cuid(CuidVersion::V2), None, None, None).unwrap();
    assert!(cuid::is_cuid2(id));
}

#[test]
fn test_ulid() {
    let id = new_id(&IDFormat::Ulid, None, None, None).unwrap();
    let parsed = ulid::Ulid::from_string(&id).unwrap();
    assert_eq!(id, parsed.to_string())
}

// ============================================
// UUID v1 Tests (Time-based)
// ============================================

#[test]
fn test_uuid_v1_simple() {
    let id = new_id(&IDFormat::Simple(UuidVersion::V1), None, None, None).unwrap();
    assert_eq!(id.len(), 32);
    assert!(!id.contains('-'));
}

#[test]
fn test_uuid_v1_hyphenated() {
    let id = new_id(&IDFormat::Hyphenated(UuidVersion::V1), None, None, None).unwrap();
    assert_eq!(id.len(), 36);
    assert_eq!(id.matches('-').count(), 4);
}

#[test]
fn test_uuid_v1_urn() {
    let id = new_id(&IDFormat::URN(UuidVersion::V1), None, None, None).unwrap();
    assert!(id.starts_with("urn:uuid:"));
    assert_eq!(id.len(), 45);
}

// ============================================
// UUID v5 Error Cases
// ============================================

#[test]
fn test_uuid_v5_requires_namespace() {
    let result = new_id(&IDFormat::Simple(UuidVersion::V5), None, None, Some("test"));
    assert!(matches!(result, Err(IDError::MissingNamespace(_))));
}

#[test]
fn test_uuid_v5_requires_name() {
    let result = new_id(
        &IDFormat::Simple(UuidVersion::V5),
        None,
        Some("6ba7b810-9dad-11d1-80b4-00c04fd430c8"),
        None,
    );
    assert!(matches!(result, Err(IDError::MissingName(_))));
}

#[test]
fn test_uuid_v3_invalid_namespace_format() {
    let result = new_id(
        &IDFormat::Simple(UuidVersion::V3),
        None,
        Some("not-a-valid-uuid"),
        Some("example.com"),
    );
    assert!(matches!(result, Err(IDError::InvalidNamespace(_))));
}

#[test]
fn test_uuid_v5_invalid_namespace_format() {
    let result = new_id(
        &IDFormat::Simple(UuidVersion::V5),
        None,
        Some("invalid-namespace"),
        Some("example.com"),
    );
    assert!(matches!(result, Err(IDError::InvalidNamespace(_))));
}

#[test]
fn test_uuid_v3_empty_namespace() {
    let result = new_id(
        &IDFormat::Simple(UuidVersion::V3),
        None,
        Some(""),
        Some("example.com"),
    );
    assert!(matches!(result, Err(IDError::InvalidNamespace(_))));
}

// ============================================
// UUID v3/v5 Deterministic Tests
// ============================================

#[test]
fn test_uuid_v3_deterministic() {
    let namespace = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
    let name = "example.com";
    let id1 = new_id(
        &IDFormat::Simple(UuidVersion::V3),
        None,
        Some(namespace),
        Some(name),
    )
    .unwrap();
    let id2 = new_id(
        &IDFormat::Simple(UuidVersion::V3),
        None,
        Some(namespace),
        Some(name),
    )
    .unwrap();
    // v3 and v5 are deterministic - same inputs = same output
    assert_eq!(id1, id2);
}

#[test]
fn test_uuid_v5_deterministic() {
    let namespace = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
    let name = "example.com";
    let id1 = new_id(
        &IDFormat::Simple(UuidVersion::V5),
        None,
        Some(namespace),
        Some(name),
    )
    .unwrap();
    let id2 = new_id(
        &IDFormat::Simple(UuidVersion::V5),
        None,
        Some(namespace),
        Some(name),
    )
    .unwrap();
    assert_eq!(id1, id2);
}

// ============================================
// NanoID Edge Cases
// ============================================

#[test]
fn test_nanoid_minimum_length() {
    let id = new_id(&IDFormat::NanoID, Some(1), None, None).unwrap();
    assert_eq!(id.len(), 1);
}

#[test]
fn test_nanoid_large_length() {
    let id = new_id(&IDFormat::NanoID, Some(100), None, None).unwrap();
    assert_eq!(id.len(), 100);
}

#[test]
fn test_nanoid_url_safe_chars() {
    let id = new_id(&IDFormat::NanoID, Some(50), None, None).unwrap();
    // NanoID uses URL-safe alphabet: A-Za-z0-9_-
    assert!(id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'));
}

// ============================================
// Uniqueness Tests
// ============================================

#[test]
fn test_uuid_v4_uniqueness() {
    let id1 = new_id(&IDFormat::Hyphenated(UuidVersion::V4), None, None, None).unwrap();
    let id2 = new_id(&IDFormat::Hyphenated(UuidVersion::V4), None, None, None).unwrap();
    assert_ne!(id1, id2);
}

#[test]
fn test_nanoid_uniqueness() {
    let id1 = new_id(&IDFormat::NanoID, None, None, None).unwrap();
    let id2 = new_id(&IDFormat::NanoID, None, None, None).unwrap();
    assert_ne!(id1, id2);
}

#[test]
fn test_ulid_uniqueness() {
    let id1 = new_id(&IDFormat::Ulid, None, None, None).unwrap();
    let id2 = new_id(&IDFormat::Ulid, None, None, None).unwrap();
    assert_ne!(id1, id2);
}

#[test]
fn test_objectid_uniqueness() {
    let id1 = new_id(&IDFormat::OID, None, None, None).unwrap();
    let id2 = new_id(&IDFormat::OID, None, None, None).unwrap();
    assert_ne!(id1, id2);
}

#[test]
fn test_cuid_v1_uniqueness() {
    let id1 = new_id(&IDFormat::Cuid(CuidVersion::V1), None, None, None).unwrap();
    let id2 = new_id(&IDFormat::Cuid(CuidVersion::V1), None, None, None).unwrap();
    assert_ne!(id1, id2);
}

#[test]
fn test_cuid_v2_uniqueness() {
    let id1 = new_id(&IDFormat::Cuid(CuidVersion::V2), None, None, None).unwrap();
    let id2 = new_id(&IDFormat::Cuid(CuidVersion::V2), None, None, None).unwrap();
    assert_ne!(id1, id2);
}

// ============================================
// Format Consistency Tests
// ============================================

#[test]
fn test_objectid_lowercase_hex() {
    let id = new_id(&IDFormat::OID, None, None, None).unwrap();
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    // ObjectId should be lowercase
    assert_eq!(id, id.to_lowercase());
}

#[test]
fn test_ulid_uppercase() {
    let id = new_id(&IDFormat::Ulid, None, None, None).unwrap();
    assert_eq!(id.len(), 26);
    // ULID uses Crockford's Base32 which is uppercase
    assert!(id
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
}
