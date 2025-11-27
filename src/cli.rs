use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

/// A lightweight, powerful CLI tool for generating and inspecting unique identifiers.
///
/// Supports UUID (v1-v5), NanoID, CUID (v1/v2), ULID, and MongoDB ObjectID.
#[derive(Parser, Debug)]
#[command(name = "idgen")]
#[command(author = "Mohamed Aamir Maniar <aamir.maniar@maniartech.com>")]
#[command(version)]
#[command(about = "Generate and inspect unique identifiers", long_about = None)]
#[command(after_help = "EXAMPLES:
    idgen                                       Generate a random UUID v4 (default)
    idgen -t uuid1                              Generate a time-based UUID v1
    idgen -t uuid3 --namespace DNS --name example.com
    idgen -t nanoid -l 10                       Generate a NanoID of length 10
    idgen -t ulid                               Generate a ULID
    idgen -c 5                                  Generate 5 UUIDs
    idgen -p 'test-' -s '.log'                  Add prefix and suffix
    idgen --json                                Output as JSON
    idgen inspect 550e8400-e29b-44d4-a716-446655440000
    idgen completions bash                      Generate bash completions")]
pub struct Cli {
    /// Type of ID to generate
    #[arg(short = 't', long = "type", value_enum, default_value = "uuid4")]
    pub id_type: IdType,

    /// Output format for UUIDs
    #[arg(short = 'f', long = "format", value_enum, default_value = "hyphenated")]
    pub format: UuidFormat,

    /// Number of IDs to generate
    #[arg(short = 'c', long = "count", default_value = "1")]
    pub count: u32,

    /// Length for NanoID (default: 21)
    #[arg(short = 'l', long = "length")]
    pub length: Option<usize>,

    /// Prefix to add to generated IDs
    #[arg(short = 'p', long = "prefix", default_value = "")]
    pub prefix: String,

    /// Suffix to add to generated IDs
    #[arg(short = 's', long = "suffix", default_value = "")]
    pub suffix: String,

    /// Namespace UUID for v3/v5 (use DNS, URL, OID, X500, or a custom UUID)
    #[arg(long = "namespace")]
    pub namespace: Option<String>,

    /// Name string for UUID v3/v5
    #[arg(long = "name")]
    pub name: Option<String>,

    /// Output as JSON
    #[arg(long = "json")]
    pub json: bool,

    /// Show banner
    #[arg(short = 'b', long = "banner")]
    pub banner: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Inspect an ID to determine its type and extract metadata
    Inspect {
        /// The ID string to inspect
        id: String,

        /// Output as JSON
        #[arg(long = "json")]
        json: bool,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Generate man page
    #[command(name = "manpage", hide = true)]
    ManPage,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum IdType {
    /// UUID version 1 (time-based)
    #[value(name = "uuid1", alias = "u1")]
    Uuid1,

    /// UUID version 3 (MD5 hash-based, requires --namespace and --name)
    #[value(name = "uuid3", alias = "u3")]
    Uuid3,

    /// UUID version 4 (random)
    #[value(name = "uuid4", alias = "u4")]
    Uuid4,

    /// UUID version 5 (SHA1 hash-based, requires --namespace and --name)
    #[value(name = "uuid5", alias = "u5")]
    Uuid5,

    /// NanoID (URL-safe, configurable length)
    #[value(name = "nanoid", alias = "nano")]
    NanoId,

    /// CUID version 1
    #[value(name = "cuid1", alias = "c1")]
    Cuid1,

    /// CUID version 2
    #[value(name = "cuid2", alias = "c2")]
    Cuid2,

    /// ULID (Universally Unique Lexicographically Sortable Identifier)
    #[value(name = "ulid")]
    Ulid,

    /// MongoDB ObjectID
    #[value(name = "objectid", alias = "oid")]
    ObjectId,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum UuidFormat {
    /// Standard hyphenated format (e.g., 550e8400-e29b-44d4-a716-446655440000)
    #[value(name = "hyphenated", alias = "h")]
    Hyphenated,

    /// Simple format without hyphens (e.g., 550e8400e29b44d4a716446655440000)
    #[value(name = "simple", alias = "s")]
    Simple,

    /// URN format (e.g., urn:uuid:550e8400-e29b-44d4-a716-446655440000)
    #[value(name = "urn", alias = "u")]
    Urn,
}

/// Well-known namespace UUIDs
pub fn resolve_namespace(namespace: &str) -> Result<String, String> {
    match namespace.to_uppercase().as_str() {
        "DNS" => Ok("6ba7b810-9dad-11d1-80b4-00c04fd430c8".to_string()),
        "URL" => Ok("6ba7b811-9dad-11d1-80b4-00c04fd430c8".to_string()),
        "OID" => Ok("6ba7b812-9dad-11d1-80b4-00c04fd430c8".to_string()),
        "X500" => Ok("6ba7b814-9dad-11d1-80b4-00c04fd430c8".to_string()),
        _ => {
            // Assume it's a custom UUID - validate format
            if namespace.len() >= 32 && (namespace.len() == 32 || namespace.len() == 36) {
                Ok(namespace.to_string())
            } else {
                Err(format!(
                    "Invalid namespace '{}'. Use DNS, URL, OID, X500, or a valid UUID.",
                    namespace
                ))
            }
        }
    }
}

pub fn build_cli() -> clap::Command {
    Cli::command()
}
