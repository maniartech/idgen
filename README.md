# idgen

Written in Rust, this tiny utility quickly generates UUID(v4), NanoID, and MongoDB ObjectIDs. This library is useful during development and testing when you need to generate UUIDs and ObjectIDs for your entities.

> ✨ The development of this library is complete and no further features are planned. We shall, however, continue to maintain the library and fix any bugs that are reported. If you have any suggestions or feedback, please feel free to open an issue or a pull request.

## Table of Contents

- [Running the Utility](#running-the-utility)
  - [Building the Binary](#building-the-binary)
  - [Usage](#usage)
- [Installation](#installation)
- [Contribution](#contribution)
- [License](#license)

## Running the Utility

Currently, we do not supply any pre-built binaries. You will need to build the binary from the source code. Please follow the instructions below to build the binary.

### Building the Binary

You will need to have [Rust](https://www.rust-lang.org/) installed on your system. Once you have Rust installed, you can build the binary using the following command:

```bash
cargo build --release
```

The binary will be created in the `target/release` directory. Copy the binary to a location in your `PATH` variable. Run the following command to verify that the binary is working:

```bash
idgen --help
```

It will print the following help information for the utility.

```txt
 _     _
(_) __| | __ _  ___ _ __
| |/ _` |/ _` |/ _ \ '_ \
| | (_| | (_| |  __/ | | |
|_|\__,_|\__, |\___|_| |_|
         |___/

ID Generator - Version 1.2.0
Mohamed Aamir Maniar - https://www.linkedin.com/in/aamironline/
Generates and prints the UUID (or ObjectID) for the specified number of times.

USAGE:
    idgen [OPTIONS]

FLAGS:
    -h --help       Prints the help information
    -v --version    Prints the version information

OPTIONS:
    -s --simple           Generates a simple UUID-v4 without hyphens
    -u --urn              Generates the UUID-v4 with URN signature
    -o --objectid         Generates the sequential MongoDB ObjectId
    -d --hyphen           Generates the hyphened version of UUID-v4 (Default)
    -n --nanoid   <num?>  Generates the nanoid with the specified length (Default: 21)
    -c --count    <num>   Number of times the ids need to be printed (Default: 1)
    -p --prefix   <str>   Prefix for the generated ids (Default: None)
```

### Usage

Here are some examples of how to use the utility:

- Generate a simple UUID-v4 without hyphens:
  ```bash
  idgen --simple
  ```

- Generate a UUID-v4 with URN signature:
  ```bash
  idgen --urn
  ```

- Generate a sequential MongoDB ObjectId:
  ```bash
  idgen --objectid
  ```

- Generate a nanoid with a specified length:
  ```bash
  idgen --nanoid 10
  ```

- Generate multiple IDs with a prefix:
  ```bash
  idgen --count 5 --prefix "ID_"
  ```

## Installation

To install Rust, follow the instructions on the [official Rust website](https://www.rust-lang.org/learn/get-started).

## Contribution

We welcome contributions! If you have any suggestions or feedback, please feel free to open an issue or a pull request. For major changes, please open an issue first to discuss what you would like to change.

## License

> **MIT License**<br>Copyright © 2021-2025 [ManiarTech](https://www.maniartech.com/). All rights reserved.