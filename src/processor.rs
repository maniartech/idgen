use std::env;

use crate::id::{new_id, IDFormat};

pub fn parse_n_process() {
    let args: Vec<String> = env::args().collect();
    let mut format = IDFormat::Hyphenated;
    let mut count = 1;
    let mut help = false;
    let mut version = false;

    args.iter().enumerate().for_each(|(i, arg)| {
        if arg == "-h" || arg == "--help" {
            help = true;
        } else if arg == "-v" || arg == "--version" {
            version = true;
        } else if arg == "-s" || arg == "--simple" {
            format = IDFormat::Simple
        } else if arg == "-u" || arg == "--urn" {
            format = IDFormat::URN
        } else if arg == "-o" || arg == "--objectid" {
            format = IDFormat::OID
        } else if arg == "-c" || arg == "--count" {
            if i < args.len() - 1 {
                count = args[i + 1].parse::<i32>().unwrap_or(1);
            }
        }
    });

    if help {
        return print_help();
    }

    if version {
        return print_version();
    }

    print_uuid(format, count);
}

fn print_uuid(id_format: IDFormat, count: i32) {
    for _ in 0..count {
        println!("{}", new_id(&id_format));
    }
}

/// Prints the program version.
fn print_version() {
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    print!("Version {}", VERSION);
}

/// Prints the help message!
fn print_help() {
    let help = String::from(
        "UUID Generator Version 0.7.0
  Mohamed Aamir Maniar - https://www.linkedin.com/in/aamironline/
  Generates and prints the UUID (or ObjectID) for the specified number of times.

  USAGE:
      uuidgen [OPTIONS]

  FLAGS:
      -h --help       Prints the help information
      -v --version    Prints the version information
      -s --simple     Generates a simple UUID-v4 without hyphen
      -u --urn        Generates the UUID-v4 with URN signature
      -o --objectid   Generates the sequential mongodb ObjectId
      -d --hyphen     Generates the hyphened version of UUID-v4 (Default)

  OPTIONS:
      -c --count <num>  Number of times the ids need to be printed (Default 1)
  ",
    )
    .replace("\n  ", "\n");
    print!("{}", help);
}
