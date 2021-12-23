use nanoid::nanoid;
use objectid::ObjectId;
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
 * Returns the newly genreated id
 */
pub fn new_id(id_format: &IDFormat, len: Option<usize>) -> String {
    match id_format {
        IDFormat::Simple => Uuid::new_v4().to_simple().to_string(),
        IDFormat::Hyphenated => Uuid::new_v4().to_string(),
        IDFormat::URN => Uuid::new_v4().to_urn().to_string(),
        IDFormat::OID => ObjectId::new().unwrap().to_string(),
        IDFormat::NanoID => {
            let l = len.unwrap();
            nanoid!(l)
        }
    }
}
