use crate::id::{new_id, CuidVersion, IDError, IDFormat, UuidVersion};
use crate::inspector::inspect_id;
use serde::Serialize;
use std::env;
use std::process;

/// Exit codes following Unix conventions
pub mod exit_codes {
    /// Successful execution
    pub const SUCCESS: i32 = 0;
    /// General runtime error (e.g., ID generation failed due to system error)
    pub const ERROR: i32 = 1;
    /// Invalid command-line arguments or usage (e.g., missing required params)
    pub const USAGE_ERROR: i32 = 2;
}

#[derive(Serialize)]
struct IdOutput {
    value: String,
}

pub fn parse_n_process() {
    let args: Vec<String> = env::args().collect();
    let mut version = UuidVersion::V4;
    let mut format = IDFormat::Hyphenated(version);
    let mut count = 1;
    let mut help = false;
    let mut show_version = false;
    let mut len: Option<usize> = None;
    let mut prefix = "";
    let mut suffix = "";
    let mut namespace: Option<String> = None;
    let mut name: Option<String> = None;
    let mut show_banner = false;
    let mut json_output = false;
    let mut inspect_target: Option<String> = None;

    let mut lastcmd = String::new();

    args.iter().enumerate().for_each(|(_, arg)| {
        if arg == "-h" || arg == "--help" {
            help = true;
        } else if arg == "-v" || arg == "--version" {
            show_version = true;
        } else if arg == "--json" {
            json_output = true;
            show_banner = false;
        } else if arg == "-s" || arg == "--simple" {
            format = IDFormat::Simple(version);
        } else if arg == "-u" || arg == "--urn" {
            format = IDFormat::URN(version);
        } else if arg == "-o" || arg == "--objectid" {
            format = IDFormat::OID;
        } else if arg == "-n" || arg == "--nano" {
            format = IDFormat::NanoID;
        } else if arg == "-c1" || arg == "--cuid1" {
            format = IDFormat::Cuid(CuidVersion::V1);
        } else if arg == "-c2" || arg == "--cuid2" {
            format = IDFormat::Cuid(CuidVersion::V2);
        } else if arg == "-l" || arg == "--ulid" {
            format = IDFormat::Ulid;
        } else if arg == "-b" || arg == "--banner" {
            show_banner = true;
        } else if arg == "-u1" || arg == "--uuid1" {
            version = UuidVersion::V1;
            format = match format.clone() {
                IDFormat::Simple(_) => IDFormat::Simple(version),
                IDFormat::Hyphenated(_) => IDFormat::Hyphenated(version),
                IDFormat::URN(_) => IDFormat::URN(version),
                _ => format.clone(),
            };
        } else if arg == "-u3" || arg == "--uuid3" {
            version = UuidVersion::V3;
            format = match format.clone() {
                IDFormat::Simple(_) => IDFormat::Simple(version),
                IDFormat::Hyphenated(_) => IDFormat::Hyphenated(version),
                IDFormat::URN(_) => IDFormat::URN(version),
                _ => format.clone(),
            };
        } else if arg == "-u4" || arg == "--uuid4" {
            version = UuidVersion::V4;
            format = match format.clone() {
                IDFormat::Simple(_) => IDFormat::Simple(version),
                IDFormat::Hyphenated(_) => IDFormat::Hyphenated(version),
                IDFormat::URN(_) => IDFormat::URN(version),
                _ => format.clone(),
            };
        } else if arg == "-u5" || arg == "--uuid5" {
            version = UuidVersion::V5;
            format = match format.clone() {
                IDFormat::Simple(_) => IDFormat::Simple(version),
                IDFormat::Hyphenated(_) => IDFormat::Hyphenated(version),
                IDFormat::URN(_) => IDFormat::URN(version),
                _ => format.clone(),
            };
        }

        if lastcmd == "-c" || lastcmd == "--count" {
            count = arg.parse::<i32>().unwrap_or(1);
        } else if lastcmd == "-n" || lastcmd == "--nano" {
            len = arg.parse::<usize>().ok();
        } else if lastcmd == "-p" || lastcmd == "--prefix" {
            prefix = arg;
        } else if lastcmd == "-f" || lastcmd == "--suffix" {
            suffix = arg;
        } else if lastcmd == "--namespace" {
            namespace = Some(arg.to_string());
        } else if lastcmd == "--name" {
            name = Some(arg.to_string());
        } else if lastcmd == "--inspect" {
            inspect_target = Some(arg.to_string());
        }

        lastcmd = arg.clone();
    });

    if let Some(target) = inspect_target {
        let result = inspect_id(&target);
        if json_output {
            let json = serde_json::to_string_pretty(&result).unwrap();
            println!("{}", json);
        } else {
            println!("ID: {}", target);
            println!("Valid: {}", result.valid);
            println!("Type: {}", result.id_type);
            if let Some(v) = result.version {
                println!("Version: {}", v);
            }
            if let Some(v) = result.variant {
                println!("Variant: {}", v);
            }
            if let Some(ts) = result.timestamp {
                println!("Timestamp: {}", ts);
            }
        }
        // Exit with error code if ID is invalid
        if !result.valid {
            process::exit(exit_codes::ERROR);
        }
        return;
    }

    if show_banner {
        print_banner();
    }

    if help {
        print_help();
        return;
    }

    if show_version {
        print_version();
        return;
    }

    // Validate count
    if count < 1 {
        eprintln!("Error: Count must be at least 1, got {}", count);
        process::exit(exit_codes::USAGE_ERROR);
    }

    match print_uuid(
        format,
        len,
        count,
        prefix,
        suffix,
        namespace.as_deref(),
        name.as_deref(),
        json_output,
    ) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Error: {}", err);
            // Check if it's a usage error (missing/invalid arguments)
            let exit_code = if err.is::<IDError>() {
                exit_codes::USAGE_ERROR
            } else {
                exit_codes::ERROR
            };
            process::exit(exit_code);
        }
    }
}

fn print_uuid(
    id_format: IDFormat,
    len: Option<usize>,
    count: i32,
    prefix: &str,
    suffix: &str,
    namespace: Option<&str>,
    name: Option<&str>,
    json_output: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if json_output {
        let mut ids = Vec::new();
        for _ in 0..count {
            let id = new_id(&id_format, len, namespace, name)?;
            ids.push(IdOutput {
                value: format!("{}{}{}", prefix, id, suffix),
            });
        }
        let json = serde_json::to_string_pretty(&ids)?;
        println!("{}", json);
    } else {
        for i in 0..count {
            let id = new_id(&id_format, len, namespace, name)?;
            print!("{}{}{}", prefix, id, suffix);
            if i < count - 1 {
                print!("\n");
            }
        }
        print!("\n");
    }
    Ok(())
}

/// Prints the program version.
fn print_version() {
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    print!("Version {}", VERSION);
}

/// Prints the help message!
fn print_help() {
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    let help = format!(
        "ID Generator Version {}
  Mohamed Aamir Maniar - https://www.linkedin.com/in/aamironline/
  Generates and prints UUIDs (all versions), NanoID, and MongoDB ObjectIDs.

  USAGE:
      idgen [OPTIONS]

  FLAGS:
      -h --help                                       Prints the help information
      -v --version                                    Prints the version information
      -b --banner                                     Show the banner output
         --json                                       Output as JSON
         --inspect <ID>                               Inspect an ID string

  UUID VERSION OPTIONS:
      -u1 --uuid1                                     Generates UUID version 1 (Time-based)
      -u3 --uuid3                                     Generates UUID version 3 (MD5 hash-based)
      -u4 --uuid4                                     Generates UUID version 4 (Random - Default)
      -u5 --uuid5                                     Generates UUID version 5 (SHA1 hash-based)

  FORMAT OPTIONS:
      -s  --simple                                     Generates UUID without hyphens
      -u  --urn                                        Generates UUID with URN signature
      -o  --objectid                                   Generates sequential MongoDB ObjectId
      -d  --hyphen                                     Generates hyphened version of UUID (Default)
      -n  --nanoid <num?>                              Generates nanoid with specified length (Default: 21)
      -c1 --cuid1                                      Generates a CUIDv1
      -c2 --cuid2                                      Generates a CUIDv2
      -l  --ulid                                       Generates a ULID

  OTHER OPTIONS:
      -c --count    <num>                             Number of IDs to generate (Default: 1)
      -p --prefix   <str>                             Prefix for the generated IDs (Default: None)
      -f --suffix   <str>                             Suffix for the generated IDs (Default: None)
         --namespace <str>                            Namespace UUID for v3/v5 (Required for v3/v5)
         --name     <str>                             Name string for v3/v5 (Required for v3/v5)

  EXAMPLES:
      idgen -u4                                       Generate a random UUID v4 (default)
      idgen -u1                                       Generate a time-based UUID v1
      idgen -u3 --namespace DNS --name example.com    Generate a v3 UUID
      idgen -u5 --namespace DNS --name example.com    Generate a v5 UUID
      idgen -s -u4                                    Generate a simple UUID v4 without hyphens
      idgen -u -u4                                    Generate a UUID v4 with URN format
      idgen -n                                        Generate a NanoID of default length (21)
      idgen -n 10                                     Generate a NanoID of length 10
      idgen -o                                        Generate a MongoDB ObjectID
      idgen -c1                                       Generate a version 1 CUID
      idgen -c2                                       Generate a version 2 CUID
      idgen -l                                        Generate a ULID
      idgen -c 5                                      Generate 5 UUIDs
      idgen -p 'test-' -c 3                           Generate 3 UUIDs with prefix 'test-'
      idgen -f '.log' -n                              Generate a NanoID with suffix '.log'
  ",
        VERSION
    )
    .replace("\n  ", "\n");
    println!("{}", help);
}

fn print_banner() {
    // represents the multiline banner text
    let banner = r#" _     _
(_) __| | __ _  ___ _ __
| |/ _` |/ _` |/ _ \ '_ \
| | (_| | (_| |  __/ | | |
|_|\__,_|\__, |\___|_| |_|
         |___/"#;
    print!("{}\n", banner); // Changed to print! and explicit \n
}
