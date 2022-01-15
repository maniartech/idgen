use std::env;

use crate::id::{new_id, IDFormat};

pub fn parse_n_process() {
    let args: Vec<String> = env::args().collect();
    let mut format = IDFormat::Hyphenated;
    let mut count = 1;
    let mut help = false;
    let mut version = false;
    let mut len: Option<usize> = None;

    let mut lastcmd = String::new();

    args.iter().enumerate().for_each(|(_, arg)| {
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
        } else if arg == "-n" || arg == "--nano" {
            format = IDFormat::NanoID
        }

        if lastcmd == "-c" || lastcmd == "--count" {
            count = arg.parse::<i32>().unwrap_or(1);
        } else if lastcmd == "-n" || lastcmd == "--nano" {
            len = Some(arg.parse::<usize>().unwrap_or(21));
        }

        lastcmd = arg.clone();
    });

    if help {
        return print_help();
    }

    if version {
        return print_version();
    }

    print_uuid(format, len, count);
}

fn print_uuid(id_format: IDFormat, len: Option<usize>, count: i32) {
    for _ in 0..count {
        println!("{}", new_id(&id_format, len));
    }
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
  Generates and prints the UUID (or ObjectID) for the specified number of times.

  USAGE:
      idgen [OPTIONS]

  FLAGS:
      -h --help       Prints the help information
      -v --version    Prints the version information

  OPTIONS:
      -s --simple           Generates a simple UUID-v4 without hyphen
      -u --urn              Generates the UUID-v4 with URN signature
      -o --objectid         Generates the sequential mongodb ObjectId
      -d --hyphen           Generates the hyphened version of UUID-v4 (Default)
      -n --nanoid   <num?>  Generates the nanoid with the specified length (Default: 21)
      -c --count    <num>   Number of times the ids need to be printed (Default 1)
  ",
        VERSION
    )
    .replace("\n  ", "\n");
    println!("{}", help);
}
