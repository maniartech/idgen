# idgen

[![GitHub release](https://img.shields.io/github/v/release/maniartech/idgen)](https://github.com/maniartech/idgen/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Edition](https://img.shields.io/badge/Rust_Edition-2021-orange.svg)](https://www.rust-lang.org/)

A lightweight command-line utility for generating and inspecting various types of unique identifiers:
- UUID (versions 1, 3, 4, and 5)
- NanoID
- CUID (versions 1 and 2)
- ULID
- MongoDB ObjectID

This tool is designed for developers who need to generate or analyze various types of IDs during development, testing, debugging, or data migration.

## Table of Contents
- [idgen](#idgen)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Installation](#installation)
    - [From crates.io (Recommended)](#from-cratesio-recommended)
    - [Pre-built Binaries](#pre-built-binaries)
    - [Build from Source](#build-from-source)
  - [Quick Start](#quick-start)
  - [Real-World Scenarios](#real-world-scenarios)
    - [Manual Database Inserts \& Seeding](#manual-database-inserts--seeding)
    - [API Testing \& Development](#api-testing--development)
    - [Scripting \& Automation](#scripting--automation)
    - [Configuration \& Secrets](#configuration--secrets)
    - [Verification \& Debugging](#verification--debugging)
    - [Distributed Tracing](#distributed-tracing)
    - [Mock Data Generation](#mock-data-generation)
    - [Cloud Resource Naming](#cloud-resource-naming)
    - [Debugging \& Inspection](#debugging--inspection)
  - [ID Types and Use Cases](#id-types-and-use-cases)
    - [UUID (Universal Unique Identifier)](#uuid-universal-unique-identifier)
      - [UUID v1 (Time-based)](#uuid-v1-time-based)
      - [UUID v4 (Random)](#uuid-v4-random)
      - [UUID v3/v5 (Name-based)](#uuid-v3v5-name-based)
    - [MongoDB ObjectID](#mongodb-objectid)
    - [NanoID](#nanoid)
    - [CUID (Collision-resistant Unique Identifier)](#cuid-collision-resistant-unique-identifier)
    - [ULID (Universally Unique Lexicographically Sortable Identifier)](#ulid-universally-unique-lexicographically-sortable-identifier)
  - [Usage Guide](#usage-guide)
    - [Command Options](#command-options)
    - [Format Options](#format-options)
    - [Examples](#examples)
    - [Common UUID Namespaces](#common-uuid-namespaces)
  - [Why idgen?](#why-idgen)
  - [Contributing](#contributing)
  - [License](#license)

## Features
- Generate UUIDs with support for all major versions (v1, v3, v4, v5)
- Create MongoDB-style ObjectIDs
- Generate URL-safe NanoIDs with configurable length
- Generate CUIDs (v1 and v2)
- Generate ULIDs
- **Inspect and identify unknown IDs** (detect type, version, and embedded timestamps)
- Multiple output formats (simple, hyphenated, URN)
- JSON output for scripting and automation
- Support for batch generation
- Custom prefix and suffix support
- **Shell completions** for bash, zsh, fish, and PowerShell
- **Man page generation** for Unix-like systems
- Banner-free mode by default (script-friendly)

## Installation

### From crates.io (Recommended)

```bash
cargo install idgen-cli
```

### Pre-built Binaries

Download the latest release for your platform from [GitHub Releases](https://github.com/maniartech/idgen/releases):

| Platform | Download |
|----------|----------|
| Linux (x64) | `idgen-linux-amd64` |
| macOS (x64) | `idgen-macos-amd64` |
| Windows (x64) | `idgen-windows-amd64.exe` |

```bash
# Linux/macOS: Make executable and move to PATH
chmod +x idgen-linux-amd64
sudo mv idgen-linux-amd64 /usr/local/bin/idgen

# Windows: Move to a directory in your PATH
```

### Build from Source

1. Install [Rust](https://www.rust-lang.org/learn/get-started) if not already installed
2. Build from source:
   ```bash
   git clone https://github.com/maniartech/idgen.git
   cd idgen
   cargo build --release
   ```
3. Copy binary to your PATH:
   ```bash
   # Linux/macOS
   cp target/release/idgen /usr/local/bin/

   # Windows (PowerShell, adjust path as needed)
   Copy-Item target/release/idgen.exe -Destination "$env:USERPROFILE/AppData/Local/Microsoft/WindowsApps/"
   ```

## Quick Start

Generate a random UUID (v4):
```bash
idgen
```

Generate with banner:
```bash
idgen -b
```

Generate multiple IDs:
```bash
idgen -c 3
```

## Real-World Scenarios

### Manual Database Inserts & Seeding

Quickly generate IDs for manual SQL `INSERT` statements or when creating seed data files (CSV/JSON) for development databases.

```bash
# Generate 5 UUIDs for a seed file
idgen -c 5
```

### API Testing & Development

Generate unique IDs on the fly when testing APIs with `curl` or Postman, especially for endpoints that require a unique `request_id` or `transaction_id`.

```bash
# Use in a curl request
curl -X POST https://api.example.com/users \
  -H "X-Request-ID: $(idgen -f simple)" \
  -d '{"name": "John"}'
```

### Scripting & Automation

Use in CI/CD pipelines or shell scripts to generate unique filenames, deployment tags, or temporary resource identifiers.

```bash
# Create a unique temporary file
touch $(idgen -t nanoid -l 10 -p temp_ -s .log)
```

### Configuration & Secrets

Generate unique strings for configuration files, such as `JWT_SECRET`, `API_KEY`, or session secrets during project setup.

```bash
# Generate a strong, random secret
idgen -t nanoid -l 64
```

### Verification & Debugging

Verify the output of deterministic IDs (like UUID v5) to ensure your application logic matches the standard.

```bash
# Verify UUID v5 generation
idgen -t uuid5 --namespace URL --name "https://example.com"
```

### Distributed Tracing

Generate a unique trace ID to manually tag a request flow across microservices when debugging.

```bash
# Generate a trace ID (UUID v4)
idgen -f simple
```

### Mock Data Generation

Generate IDs for JSON mock files used in frontend development.

```bash
# Generate 10 NanoIDs for a mock user list
idgen -t nanoid -c 10

# Generate mock email addresses
idgen -t nanoid -l 8 -s @example.com -c 5
```

### Cloud Resource Naming

Generate unique tags for cloud resources (AWS/Azure/GCP) during manual provisioning or Terraform/Ansible runs.

```bash
# Generate a unique suffix for an S3 bucket
idgen -t nanoid -l 8 -p my-bucket- | tr '[:upper:]' '[:lower:]'
```

### Debugging & Inspection

Analyze unknown IDs found in logs or databases to determine their type, version, and creation timestamp (if available).

```bash
# Inspect an ID to see what it is
idgen inspect 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Check if an ID is valid with JSON output
idgen inspect 550e8400-e29b-44d4-a716-446655440000 --json
```

## ID Types and Use Cases

### UUID (Universal Unique Identifier)
Standard 128-bit identifiers with multiple versions for different needs:

#### UUID v1 (Time-based)
- Format: Timestamp + node ID based
- Example: `550e8400-e29b-11d4-a716-446655440000`
- Best for: Logging, temporal ordering, distributed systems

#### UUID v4 (Random)
- Format: Random numbers
- Example: `550e8400-e29b-44d4-a716-446655440000`
- Best for: Default choice, database keys, session IDs

#### UUID v3/v5 (Name-based)
- v3 uses MD5, v5 uses SHA-1 (preferred)
- Example: `cfbff0d1-9375-5685-968c-48ce8b15ae17`
- Best for: Consistent IDs from same input, content addressing

### MongoDB ObjectID
12-byte identifier combining timestamp, machine ID, and counter:
- Example: `507f1f77bcf86cd799439011`
- Best for: Document IDs, chronological sorting

### NanoID
Compact, URL-safe identifiers:
- Example: `V1StGXR8_Z5jdHi6B-myT`
- Best for: Short URLs, user-facing IDs, frontend generation

### CUID (Collision-resistant Unique Identifier)
Designed for horizontal scaling and performance:
- v1: Original version
- v2: Secure, collision-resistant, optimized for modern web
- Best for: High-performance distributed systems

### ULID (Universally Unique Lexicographically Sortable Identifier)
Sortable, random, 128-bit identifier:
- Example: `01ARZ3NDEKTSV4RRFFQ69G5FAV`
- Best for: Database keys where sorting is important

## Usage Guide

### Command Options

```
Generate and inspect unique identifiers

Usage: idgen.exe [OPTIONS] [COMMAND]

Commands:
  inspect      Inspect an ID to determine its type and extract metadata
  completions  Generate shell completions
  help         Print this message or the help of the given subcommand(s)

Options:
  -t, --type <ID_TYPE>
          Type of ID to generate

          Possible values:
          - uuid1:    UUID version 1 (time-based)
          - uuid3:    UUID version 3 (MD5 hash-based, requires --namespace and --name)
          - uuid4:    UUID version 4 (random)
          - uuid5:    UUID version 5 (SHA1 hash-based, requires --namespace and --name)
          - nanoid:   NanoID (URL-safe, configurable length)
          - cuid1:    CUID version 1
          - cuid2:    CUID version 2
          - ulid:     ULID (Universally Unique Lexicographically Sortable Identifier)
          - objectid: MongoDB ObjectID

          [default: uuid4]

  -f, --format <FORMAT>
          Output format for UUIDs

          Possible values:
          - hyphenated: Standard hyphenated format (e.g., 550e8400-e29b-44d4-a716-446655440000)
          - simple:     Simple format without hyphens (e.g., 550e8400e29b44d4a716446655440000)
          - urn:        URN format (e.g., urn:uuid:550e8400-e29b-44d4-a716-446655440000)

          [default: hyphenated]

  -c, --count <COUNT>
          Number of IDs to generate

          [default: 1]

  -l, --length <LENGTH>
          Length for NanoID (default: 21)

  -p, --prefix <PREFIX>
          Prefix to add to generated IDs

          [default: ]

  -s, --suffix <SUFFIX>
          Suffix to add to generated IDs

          [default: ]

      --namespace <NAMESPACE>
          Namespace UUID for v3/v5 (use DNS, URL, OID, X500, or a custom UUID)

      --name <NAME>
          Name string for UUID v3/v5

      --json
          Output as JSON

  -b, --banner
          Show banner

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

EXAMPLES:
    idgen                                       Generate a random UUID v4 (default)
    idgen -t uuid1                              Generate a time-based UUID v1
    idgen -t uuid3 --namespace DNS --name example.com
    idgen -t nanoid -l 10                       Generate a NanoID of length 10
    idgen -t ulid                               Generate a ULID
    idgen -c 5                                  Generate 5 UUIDs
    idgen -p 'test-' -s '.log'                  Add prefix and suffix
    idgen --json                                Output as JSON
    idgen inspect 550e8400-e29b-44d4-a716-446655440000
    idgen completions bash                      Generate bash completions
```

### Format Options
Each ID can be formatted in different ways:

- **Simple**: No separators (`550e8400e29b44d4a716446655440000`)
- **Hyphenated**: Standard format (`550e8400-e29b-44d4-a716-446655440000`)
- **URN**: URN format (`urn:uuid:550e8400-e29b-44d4-a716-446655440000`)

### Examples

```bash
# Generate IDs (default: UUID v4)
idgen                              # Random UUID v4
idgen -t uuid1                     # Time-based UUID v1
idgen -t uuid5 --namespace DNS --name example.com  # Name-based UUID v5

# ID Types
idgen -t nanoid                    # NanoID (21 chars)
idgen -t nanoid -l 10              # NanoID with custom length
idgen -t cuid1                     # CUID v1
idgen -t cuid2                     # CUID v2
idgen -t ulid                      # ULID
idgen -t objectid                  # MongoDB ObjectID

# UUID Formats
idgen -f simple                    # No hyphens: 550e8400e29b44d4a716446655440000
idgen -f urn                       # URN format: urn:uuid:550e8400-e29b-...

# Multiple IDs
idgen -c 5                         # Generate 5 UUIDs
idgen -t ulid -c 3                 # Generate 3 ULIDs

# Prefix and Suffix
idgen -p 'test-'                   # Add prefix
idgen -s '.log'                    # Add suffix
idgen -p 'user_' -s '_v1' -c 3     # Both prefix and suffix

# JSON Output
idgen --json                       # Single ID as JSON
idgen --json -c 3                  # Multiple IDs as JSON array

# Inspect IDs
idgen inspect 550e8400-e29b-44d4-a716-446655440000
idgen inspect 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Shell Completions
idgen completions bash > ~/.bash_completion.d/idgen
idgen completions zsh > ~/.zsh/completions/_idgen
idgen completions fish > ~/.config/fish/completions/idgen.fish
idgen completions powershell >> $PROFILE

# Man Page
idgen manpage > /usr/local/share/man/man1/idgen.1
```

### Common UUID Namespaces
For UUID v3/v5, use these standard namespaces:
- DNS: `6ba7b810-9dad-11d1-80b4-00c04fd430c8`
- URL: `6ba7b811-9dad-11d1-80b4-00c04fd430c8`
- OID: `6ba7b812-9dad-11d1-80b4-00c04fd430c8`
- X500: `6ba7b814-9dad-11d1-80b4-00c04fd430c8`

## Why idgen?

| Feature | idgen | uuidgen | uuid (npm) | nanoid (npm) |
|---------|-------|---------|------------|--------------|
| UUID v1-v5 | ✅ | ✅ | ✅ | ❌ |
| NanoID | ✅ | ❌ | ❌ | ✅ |
| CUID v1/v2 | ✅ | ❌ | ❌ | ❌ |
| ULID | ✅ | ❌ | ❌ | ❌ |
| ObjectID | ✅ | ❌ | ❌ | ❌ |
| ID Inspection | ✅ | ❌ | ❌ | ❌ |
| Shell Completions | ✅ | ❌ | ❌ | ❌ |
| Zero Runtime Deps | ✅ | ✅ | ❌ | ❌ |
| Single Binary | ✅ | ✅ | ❌ | ❌ |

**Key advantages:**
- **All-in-one**: Single tool for 9 ID types instead of multiple utilities
- **ID Inspector**: Unique feature to analyze and identify unknown IDs
- **Fast**: Native Rust binary with no interpreter overhead
- **Portable**: No Node.js, Python, or other runtime required

## Contributing

We welcome contributions! Please feel free to submit a Pull Request. For major changes, open an issue first.

## License

MIT License - Copyright © 2021-2025 [ManiarTech®](https://www.maniartech.com/)