use crate::cli::{build_cli, resolve_namespace, Cli, Commands, IdType, UuidFormat};
use crate::id::{new_id, CuidVersion, IDError, IDFormat, UuidVersion};
use crate::inspector::inspect_id;
use clap::Parser;
use clap_complete::generate;
use clap_mangen::Man;
use serde::Serialize;
use std::io;
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
    let cli = Cli::parse();

    // Handle subcommands first
    if let Some(command) = &cli.command {
        match command {
            Commands::Inspect { id, json } => {
                handle_inspect(id, *json);
                return;
            }
            Commands::Completions { shell } => {
                let mut cmd = build_cli();
                generate(*shell, &mut cmd, "idgen", &mut io::stdout());
                return;
            }
            Commands::ManPage => {
                let cmd = build_cli();
                let man = Man::new(cmd);
                let mut buffer: Vec<u8> = Vec::new();
                man.render(&mut buffer)
                    .expect("Failed to generate man page");
                print!("{}", String::from_utf8_lossy(&buffer));
                return;
            }
        }
    }

    // Show banner if requested
    if cli.banner {
        print_banner();
    }

    // Validate count
    if cli.count < 1 {
        eprintln!("Error: Count must be at least 1, got {}", cli.count);
        process::exit(exit_codes::USAGE_ERROR);
    }

    // Convert CLI options to internal types
    let (id_format, namespace, name) = match build_id_format(&cli) {
        Ok(result) => result,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            process::exit(exit_codes::USAGE_ERROR);
        }
    };

    // Generate IDs
    match generate_ids(&id_format, &cli, namespace.as_deref(), name.as_deref()) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Error: {}", err);
            let exit_code = if err.is::<IDError>() {
                exit_codes::USAGE_ERROR
            } else {
                exit_codes::ERROR
            };
            process::exit(exit_code);
        }
    }
}

fn handle_inspect(id: &str, json_output: bool) {
    let result = inspect_id(id);

    if json_output {
        let json = serde_json::to_string_pretty(&result).unwrap();
        println!("{}", json);
    } else {
        println!("ID: {}", id);
        println!("Valid: {}", result.valid);
        println!("Type: {}", result.id_type);
        if let Some(v) = &result.version {
            println!("Version: {}", v);
        }
        if let Some(v) = &result.variant {
            println!("Variant: {}", v);
        }
        if let Some(ts) = &result.timestamp {
            println!("Timestamp: {}", ts);
        }
    }

    if !result.valid {
        process::exit(exit_codes::ERROR);
    }
}

fn build_id_format(cli: &Cli) -> Result<(IDFormat, Option<String>, Option<String>), String> {
    let uuid_version = match cli.id_type {
        IdType::Uuid1 => Some(UuidVersion::V1),
        IdType::Uuid3 => Some(UuidVersion::V3),
        IdType::Uuid4 => Some(UuidVersion::V4),
        IdType::Uuid5 => Some(UuidVersion::V5),
        _ => None,
    };

    // Handle namespace resolution for v3/v5
    let namespace = if matches!(cli.id_type, IdType::Uuid3 | IdType::Uuid5) {
        match &cli.namespace {
            Some(ns) => match resolve_namespace(ns) {
                Ok(resolved) => Some(resolved),
                Err(e) => return Err(e),
            },
            None => {
                return Err(format!(
                    "UUID {} requires --namespace parameter. Use DNS, URL, OID, X500, or a custom UUID.",
                    if cli.id_type == IdType::Uuid3 { "v3" } else { "v5" }
                ));
            }
        }
    } else {
        None
    };

    let name = if matches!(cli.id_type, IdType::Uuid3 | IdType::Uuid5) {
        match &cli.name {
            Some(n) => Some(n.clone()),
            None => {
                return Err(format!(
                    "UUID {} requires --name parameter.",
                    if cli.id_type == IdType::Uuid3 {
                        "v3"
                    } else {
                        "v5"
                    }
                ));
            }
        }
    } else {
        None
    };

    let format = match cli.id_type {
        IdType::Uuid1 | IdType::Uuid3 | IdType::Uuid4 | IdType::Uuid5 => {
            let version = uuid_version.unwrap();
            match cli.format {
                UuidFormat::Simple => IDFormat::Simple(version),
                UuidFormat::Hyphenated => IDFormat::Hyphenated(version),
                UuidFormat::Urn => IDFormat::URN(version),
            }
        }
        IdType::NanoId => IDFormat::NanoID,
        IdType::Cuid1 => IDFormat::Cuid(CuidVersion::V1),
        IdType::Cuid2 => IDFormat::Cuid(CuidVersion::V2),
        IdType::Ulid => IDFormat::Ulid,
        IdType::ObjectId => IDFormat::OID,
    };

    Ok((format, namespace, name))
}

fn generate_ids(
    id_format: &IDFormat,
    cli: &Cli,
    namespace: Option<&str>,
    name: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let len = cli.length;

    if cli.json {
        let mut ids = Vec::new();
        for _ in 0..cli.count {
            let id = new_id(id_format, len, namespace, name)?;
            ids.push(IdOutput {
                value: format!("{}{}{}", cli.prefix, id, cli.suffix),
            });
        }
        let json = serde_json::to_string_pretty(&ids)?;
        println!("{}", json);
    } else {
        for i in 0..cli.count {
            let id = new_id(id_format, len, namespace, name)?;
            print!("{}{}{}", cli.prefix, id, cli.suffix);
            if i < cli.count - 1 {
                println!();
            }
        }
        println!();
    }

    Ok(())
}

fn print_banner() {
    let banner = r#" _     _
(_) __| | __ _  ___ _ __
| |/ _` |/ _` |/ _ \ '_ \
| | (_| | (_| |  __/ | | |
|_|\__,_|\__, |\___|_| |_|
         |___/"#;
    println!("{}", banner);
}
