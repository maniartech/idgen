use idgen::inspector::inspect_id;

// ============================================
// UUID Detection Tests
// ============================================

#[test]
fn test_inspect_uuid_v4() {
    let result = inspect_id("550e8400-e29b-44d4-a716-446655440000");
    assert!(result.valid);
    assert_eq!(result.id_type, "UUID");
    assert_eq!(result.version, Some("Random".to_string()));
    assert_eq!(result.variant, Some("RFC4122".to_string()));
}

#[test]
fn test_inspect_uuid_v1() {
    // UUID v1 example
    let result = inspect_id("f47ac10b-58cc-11e4-8b58-0800200c9a66");
    assert!(result.valid);
    assert_eq!(result.id_type, "UUID");
    assert_eq!(result.version, Some("Mac".to_string()));
}

#[test]
fn test_inspect_uuid_simple_format() {
    // UUID without hyphens
    let result = inspect_id("550e8400e29b44d4a716446655440000");
    assert!(result.valid);
    assert_eq!(result.id_type, "UUID");
}

#[test]
fn test_inspect_uuid_urn_format() {
    // UUID with URN prefix
    let result = inspect_id("urn:uuid:550e8400-e29b-44d4-a716-446655440000");
    assert!(result.valid);
    assert_eq!(result.id_type, "UUID");
}

#[test]
fn test_inspect_uuid_uppercase() {
    let result = inspect_id("550E8400-E29B-44D4-A716-446655440000");
    assert!(result.valid);
    assert_eq!(result.id_type, "UUID");
}

// ============================================
// ULID Detection Tests
// ============================================

#[test]
fn test_inspect_ulid() {
    let result = inspect_id("01ARZ3NDEKTSV4RRFFQ69G5FAV");
    assert!(result.valid);
    assert_eq!(result.id_type, "ULID");
    assert!(result.timestamp.is_some());
}

#[test]
fn test_inspect_ulid_lowercase() {
    // ULID should work with lowercase too
    let result = inspect_id("01arz3ndektsv4rrffq69g5fav");
    assert!(result.valid);
    assert_eq!(result.id_type, "ULID");
}

// ============================================
// ObjectId Detection Tests
// ============================================

#[test]
fn test_inspect_objectid() {
    let result = inspect_id("507f1f77bcf86cd799439011");
    assert!(result.valid);
    assert_eq!(result.id_type, "ObjectId");
    assert!(result.timestamp.is_some());
}

#[test]
fn test_inspect_objectid_uppercase() {
    let result = inspect_id("507F1F77BCF86CD799439011");
    assert!(result.valid);
    assert_eq!(result.id_type, "ObjectId");
}

// ============================================
// CUID Detection Tests
// ============================================

#[test]
fn test_inspect_cuid_v1() {
    // CUID v1 starts with 'c' and is 25+ chars
    let result = inspect_id("clh3am2f10000qwer1234abcde");
    assert!(result.valid);
    assert_eq!(result.id_type, "CUID");
    assert_eq!(result.version, Some("v1".to_string()));
}

#[test]
fn test_inspect_cuid_v2() {
    // CUID v2 is 24 lowercase alphanumeric chars
    let result = inspect_id("abcdefghij0123456789abcd");
    assert!(result.valid);
    assert_eq!(result.id_type, "CUID");
    assert_eq!(result.version, Some("v2".to_string()));
}

// ============================================
// NanoID Detection Tests
// ============================================

#[test]
fn test_inspect_nanoid() {
    let result = inspect_id("V1StGXR8_Z5jdHi6B-myT");
    assert!(result.valid);
    assert_eq!(result.id_type, "NanoID");
}

#[test]
fn test_inspect_nanoid_with_underscore() {
    let result = inspect_id("____________________-");
    assert!(result.valid);
    assert_eq!(result.id_type, "NanoID");
}

#[test]
fn test_inspect_nanoid_with_dash() {
    let result = inspect_id("---------------------");
    assert!(result.valid);
    assert_eq!(result.id_type, "NanoID");
}

// ============================================
// Unknown/Invalid Detection Tests
// ============================================

#[test]
fn test_inspect_unknown() {
    let result = inspect_id("not-a-valid-id");
    assert!(!result.valid);
    assert_eq!(result.id_type, "Unknown");
}

#[test]
fn test_inspect_empty_string() {
    let result = inspect_id("");
    assert!(!result.valid);
    assert_eq!(result.id_type, "Unknown");
}

#[test]
fn test_inspect_whitespace_only() {
    let result = inspect_id("   ");
    assert!(!result.valid);
    assert_eq!(result.id_type, "Unknown");
}

#[test]
fn test_inspect_special_characters() {
    let result = inspect_id("!@#$%^&*()");
    assert!(!result.valid);
    assert_eq!(result.id_type, "Unknown");
}

#[test]
fn test_inspect_very_long_string() {
    let long_string = "a".repeat(1000);
    let result = inspect_id(&long_string);
    assert!(!result.valid);
    assert_eq!(result.id_type, "Unknown");
}

#[test]
fn test_inspect_single_character() {
    let result = inspect_id("a");
    assert!(!result.valid);
    assert_eq!(result.id_type, "Unknown");
}

// ============================================
// Malformed UUID Tests
// ============================================

#[test]
fn test_inspect_uuid_wrong_length() {
    // Too short
    let result = inspect_id("550e8400-e29b-44d4-a716");
    assert!(!result.valid);
    assert_eq!(result.id_type, "Unknown");
}

#[test]
fn test_inspect_uuid_invalid_chars() {
    // 'g' is not a valid hex character
    let result = inspect_id("550g8400-e29b-44d4-a716-446655440000");
    assert!(!result.valid);
    assert_eq!(result.id_type, "Unknown");
}

#[test]
fn test_inspect_uuid_wrong_hyphen_positions() {
    let result = inspect_id("550e84-00e29b-44d4-a716-446655440000");
    assert!(!result.valid);
    assert_eq!(result.id_type, "Unknown");
}

// ============================================
// Edge Cases for ObjectId
// ============================================

#[test]
fn test_inspect_objectid_23_chars() {
    // Too short for ObjectId (needs exactly 24)
    let result = inspect_id("507f1f77bcf86cd79943901");
    assert!(!result.valid);
    assert_eq!(result.id_type, "Unknown");
}

#[test]
fn test_inspect_objectid_25_chars() {
    // Too long for ObjectId
    let result = inspect_id("507f1f77bcf86cd7994390111");
    // This is 25 chars, might match CUID v1 if starts with 'c'
    // Since it starts with '5', it's unknown
    assert!(!result.valid);
    assert_eq!(result.id_type, "Unknown");
}

// ============================================
// Numeric Input Tests
// ============================================

#[test]
fn test_inspect_numeric_only() {
    let result = inspect_id("12345678901234567890");
    assert!(!result.valid);
    assert_eq!(result.id_type, "Unknown");
}
