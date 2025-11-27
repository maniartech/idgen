use chrono::{DateTime, TimeZone, Utc};
use regex::Regex;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Debug)]
pub struct InspectionResult {
    pub valid: bool,
    pub id_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

pub fn inspect_id(id: &str) -> InspectionResult {
    // 1. Try UUID
    if let Ok(uuid) = Uuid::parse_str(id) {
        let version = uuid.get_version().map(|v| format!("{:?}", v));
        let variant = format!("{:?}", uuid.get_variant());

        // Extract timestamp for v1 and v7 (if supported by crate, v1 is standard)
        // Note: uuid crate v1.0+ supports getting timestamp from v1, v6, v7
        let timestamp = if let Some(uuid::Version::Mac) = uuid.get_version() {
            // UUID v1 timestamp extraction is complex without direct crate support in older versions
            // For now, we'll skip complex timestamp extraction for UUIDs to keep it simple
            // unless we upgrade to uuid v1.0+ features explicitly.
            // Actually, let's try a best effort for v1 if the crate allows,
            // but the current uuid crate version in Cargo.toml is 1.18.1 which is good.

            // uuid 1.x exposes get_timestamp() which returns a Timestamp struct
            uuid.get_timestamp().and_then(|ts| {
                let (secs, nanos) = ts.to_unix();
                Utc.timestamp_opt(secs as i64, nanos)
                    .single()
                    .map(|dt| dt.to_rfc3339())
            })
        } else {
            None
        };

        return InspectionResult {
            valid: true,
            id_type: "UUID".to_string(),
            version,
            timestamp,
            variant: Some(variant),
        };
    }

    // 2. Try ULID
    if let Ok(ulid) = ulid::Ulid::from_string(id) {
        let datetime: DateTime<Utc> = ulid.datetime().into();
        return InspectionResult {
            valid: true,
            id_type: "ULID".to_string(),
            version: None,
            timestamp: Some(datetime.to_rfc3339()),
            variant: None,
        };
    }

    // 3. Try MongoDB ObjectId (24 hex chars)
    let object_id_regex = Regex::new(r"^[0-9a-fA-F]{24}$").unwrap();
    if object_id_regex.is_match(id) {
        // Extract timestamp (first 4 bytes / 8 hex chars)
        if let Ok(timestamp_hex) = u32::from_str_radix(&id[0..8], 16) {
            let datetime = Utc.timestamp_opt(timestamp_hex as i64, 0).single();
            return InspectionResult {
                valid: true,
                id_type: "ObjectId".to_string(),
                version: None,
                timestamp: datetime.map(|dt| dt.to_rfc3339()),
                variant: None,
            };
        }
    }

    // 4. Try CUID (v1 starts with 'c', v2 is 24 chars usually)
    // CUID v1
    if id.starts_with('c') && id.len() >= 25 {
        return InspectionResult {
            valid: true,
            id_type: "CUID".to_string(),
            version: Some("v1".to_string()),
            timestamp: None, // CUID v1 timestamp is base36 encoded, doable but custom logic
            variant: None,
        };
    }

    // CUID v2 (24 chars, usually starts with lowercase letter)
    // This is a weak heuristic, so we put it last-ish
    if id.len() == 24
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    {
        // Could be CUID v2 or just a random string.
        // Since ObjectId is also 24 hex, we checked that first.
        // If it wasn't hex, it might be CUID v2.
        return InspectionResult {
            valid: true,
            id_type: "CUID".to_string(),
            version: Some("v2".to_string()),
            timestamp: None,
            variant: None,
        };
    }

    // 5. NanoID (Hard to detect definitively as it's just random chars)
    // We can just check for URL-safe chars and length
    let nanoid_regex = Regex::new(r"^[A-Za-z0-9_-]{21}$").unwrap();
    if nanoid_regex.is_match(id) {
        return InspectionResult {
            valid: true,
            id_type: "NanoID".to_string(),
            version: None,
            timestamp: None,
            variant: None,
        };
    }

    InspectionResult {
        valid: false,
        id_type: "Unknown".to_string(),
        version: None,
        timestamp: None,
        variant: None,
    }
}
