use crate::id::{new_id, CuidVersion, IDFormat, UuidVersion};
use std::env;
use std::process;

pub fn parse_n_process() {
    let args: Vec<String> = env::args().collect();
    let mut version = UuidVersion::V4;
    let mut format = IDFormat::Hyphenated(version);
    let mut count = 1;
    let mut help = false;
    let mut show_version = false;
    let mut len: Option<usize> = None;
    let mut prefix = "";
    let mut namespace: Option<String> = None;
    let mut name: Option<String> = None;
    let mut show_banner = true;

    let mut lastcmd = String::new();

    args.iter().enumerate().for_each(|(_, arg)| {
        if arg == "-h" || arg == "--help" {
            help = true;
        } else if arg == "-v" || arg == "--version" {
            show_version = true;
        } else if arg == "-s" || arg == "--simple" {
            format = IDFormat::Simple(version);
        } else if arg == "-u" || arg == "--urn" {
            format = IDFormat::URN(version);
        } else if arg == "-o" || arg == "--objectid" {
            format = IDFormat::OID;
        } else if arg == "-n" || arg == "--nano" {
            format = IDFormat::NanoID;
        } else if arg == "-C" || arg == "--cuid" {
            format = IDFormat::Cuid(CuidVersion::V1);
        } else if arg == "-U" || arg == "--cuid2" {
            format = IDFormat::Cuid(CuidVersion::V2);
        } else if arg == "-l" || arg == "--ulid" {
            format = IDFormat::Ulid;
        } else if arg == "-nb" || arg == "--no-banner" {
            show_banner = false;
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
            len = Some(arg.parse::<usize>().unwrap_or(21));
        } else if lastcmd == "-p" || lastcmd == "--prefix" {
            prefix = arg;
        } else if lastcmd == "--namespace" {
            namespace = Some(arg.to_string());
        } else if lastcmd == "--name" {
            name = Some(arg.to_string());
        }

        lastcmd = arg.clone();
    });

    if show_banner {
        print_banner();
    }

    if help {
        return print_help();
    }

    if show_version {
        return print_version();
    }

    if let Err(err) = print_uuid(
        format,
        len,
        count,
        prefix,
        namespace.as_deref(),
        name.as_deref(),
    ) {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

fn print_uuid(
    id_format: IDFormat,
    len: Option<usize>,
    count: i32,
    prefix: &str,
    namespace: Option<&str>,
    name: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    for i in 0..count {
        let id = new_id(&id_format, len, namespace, name)?;
        print!("{}{}", prefix, id);
        if i < count - 1 {
            print!("\n");
        }
    }
    print!("\n");
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
      -nb --no-banner                                 Suppresses the banner output

  UUID VERSION OPTIONS:
      -u1 --uuid1                                     Generates UUID version 1 (Time-based)
      -u3 --uuid3                                     Generates UUID version 3 (MD5 hash-based)
      -u4 --uuid4                                     Generates UUID version 4 (Random - Default)
      -u5 --uuid5                                     Generates UUID version 5 (SHA1 hash-based)

  FORMAT OPTIONS:
      -s --simple                                     Generates UUID without hyphens
      -u --urn                                        Generates UUID with URN signature
      -o --objectid                                   Generates sequential MongoDB ObjectId
      -d --hyphen                                     Generates hyphened version of UUID (Default)
      -n --nanoid <num?>                              Generates nanoid with specified length (Default: 21)
      -C --cuid                                       Generates a CUIDv1
      -U --cuid2                                      Generates a CUIDv2
      -l --ulid                                       Generates a ULID

  OTHER OPTIONS:
      -c --count    <num>                             Number of IDs to generate (Default: 1)
      -p --prefix   <str>                             Prefix for the generated IDs (Default: None)
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
      idgen -C                                        Generate a version 1 CUID
      idgen -U                                        Generate a version 2 CUID
      idgen -l                                        Generate a ULID
      idgen -c 5                                      Generate 5 UUIDs
      idgen -p 'test-' -c 3                           Generate 3 UUIDs with prefix 'test-'
      idgen -nb -u4                                   Generate a UUID v4 without banner
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

#[cfg(test)]
mod tests {
    use super::*;

    fn with_args(args: Vec<&str>) -> Vec<String> {
        let mut full_args = vec!["program"];
        full_args.extend(args);
        full_args.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_default_format() {
        let args = with_args(vec![]);
        let version = UuidVersion::V4;
        let mut format = IDFormat::Hyphenated(version);
        let mut count = 1;
        let mut help = false;
        let mut show_version = false;
        let mut len: Option<usize> = None;
        let mut prefix = "";
        let mut namespace: Option<String> = None;
        let mut name: Option<String> = None;
        let mut lastcmd = String::new();

        args.iter().enumerate().for_each(|(_, arg)| {
            if arg == "-h" || arg == "--help" {
                help = true;
            } else if arg == "-v" || arg == "--version" {
                show_version = true;
            } else if arg == "-s" || arg == "--simple" {
                format = IDFormat::Simple(version);
            } else if arg == "-u" || arg == "--urn" {
                format = IDFormat::URN(version);
            } else if arg == "-o" || arg == "--objectid" {
                format = IDFormat::OID;
            } else if arg == "-n" || arg == "--nano" {
                format = IDFormat::NanoID;
            }

            if lastcmd == "-c" || lastcmd == "--count" {
                count = arg.parse::<i32>().unwrap_or(1);
            } else if lastcmd == "-n" || lastcmd == "--nano" {
                len = Some(arg.parse::<usize>().unwrap_or(21));
            } else if lastcmd == "-p" || lastcmd == "--prefix" {
                prefix = arg;
            } else if lastcmd == "--namespace" {
                namespace = Some(arg.to_string());
            } else if lastcmd == "--name" {
                name = Some(arg.to_string());
            }

            lastcmd = arg.clone();
        });

        assert!(matches!(format, IDFormat::Hyphenated(_)));
        assert_eq!(count, 1);
        assert!(!help);
        assert!(!show_version);
        assert_eq!(len, None);
        assert_eq!(prefix, "");
        assert_eq!(namespace, None);
        assert_eq!(name, None);
    }

    #[test]
    fn test_simple_format() {
        let args = with_args(vec!["--simple"]);
        let version = UuidVersion::V4;
        let mut format = IDFormat::Hyphenated(version);
        let mut lastcmd = String::new();

        args.iter().enumerate().for_each(|(_, arg)| {
            if arg == "-s" || arg == "--simple" {
                format = IDFormat::Simple(version);
            }
            lastcmd = arg.clone();
        });

        assert!(matches!(format, IDFormat::Simple(_)));
    }

    #[test]
    fn test_uuid_version() {
        let args = with_args(vec!["--uuid3"]);
        let mut version = UuidVersion::V4;
        let mut format = IDFormat::Hyphenated(version);
        let mut lastcmd = String::new();

        args.iter().enumerate().for_each(|(_, arg)| {
            if arg == "-u3" || arg == "--uuid3" {
                version = UuidVersion::V3;
                format = IDFormat::Hyphenated(version);
            }
            lastcmd = arg.clone();
        });

        assert!(matches!(format, IDFormat::Hyphenated(UuidVersion::V3)));
    }

    // Existing tests remain unchanged
    #[test]
    fn test_count_option() {
        let args = with_args(vec!["--count", "5"]);
        let mut count = 1;
        let mut lastcmd = String::new();

        args.iter().enumerate().for_each(|(_, arg)| {
            if lastcmd == "-c" || lastcmd == "--count" {
                count = arg.parse::<i32>().unwrap_or(1);
            }
            lastcmd = arg.clone();
        });

        assert_eq!(count, 5);
    }

    #[test]
    fn test_nanoid_length() {
        let args = with_args(vec!["--nano", "10"]);
        let version = UuidVersion::V4;
        let mut format = IDFormat::Hyphenated(version);
        let mut len: Option<usize> = None;
        let mut lastcmd = String::new();

        args.iter().enumerate().for_each(|(_, arg)| {
            if arg == "-n" || arg == "--nano" {
                format = IDFormat::NanoID;
            } else if lastcmd == "-n" || lastcmd == "--nano" {
                len = Some(arg.parse::<usize>().unwrap_or(21));
            }
            lastcmd = arg.clone();
        });

        assert!(matches!(format, IDFormat::NanoID));
        assert_eq!(len, Some(10));
    }

    #[test]
    fn test_prefix_option() {
        let args = with_args(vec!["--prefix", "test-"]);
        let mut prefix = "";
        let mut lastcmd = String::new();

        args.iter().enumerate().for_each(|(_, arg)| {
            if lastcmd == "-p" || lastcmd == "--prefix" {
                prefix = arg;
            }
            lastcmd = arg.clone();
        });

        assert_eq!(prefix, "test-");
    }

    #[test]
    fn test_uuid_v3_parameters() {
        let args = with_args(vec![
            "--uuid3",
            "--namespace",
            "DNS",
            "--name",
            "example.com",
        ]);
        let mut version = UuidVersion::V4;
        let mut format = IDFormat::Hyphenated(version);
        let mut namespace: Option<String> = None;
        let mut name: Option<String> = None;
        let mut lastcmd = String::new();

        args.iter().enumerate().for_each(|(_, arg)| {
            if arg == "-u3" || arg == "--uuid3" {
                version = UuidVersion::V3;
                format = IDFormat::Hyphenated(version);
            } else if lastcmd == "--namespace" {
                namespace = Some(arg.to_string());
            } else if lastcmd == "--name" {
                name = Some(arg.to_string());
            }
            lastcmd = arg.clone();
        });

        assert!(matches!(format, IDFormat::Hyphenated(UuidVersion::V3)));
        assert_eq!(namespace, Some("DNS".to_string()));
        assert_eq!(name, Some("example.com".to_string()));
    }

    #[test]
    fn test_no_banner_flag() {
        let args = with_args(vec!["--no-banner"]);
        let mut show_banner = true;

        args.iter().enumerate().for_each(|(_, arg)| {
            if arg == "-nb" || arg == "--no-banner" {
                show_banner = false;
            }
        });

        assert!(!show_banner);
    }
}
