# idgen

A lightweight command-line utility for generating various types of unique identifiers:
- UUID (versions 1, 3, 4, and 5)
- NanoID
- CUID (versions 1 and 2)
- ULID
- MongoDB ObjectID

This tool is designed for developers who need to generate various types of IDs during development, testing, or data migration.

## Table of Contents
- [idgen](#idgen)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Installation](#installation)
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
  - [Contributing](#contributing)
  - [License](#license)

## Features
- Generate UUIDs with support for all major versions (v1, v3, v4, v5)
- Create MongoDB-style ObjectIDs
- Generate URL-safe NanoIDs with configurable length
- Generate CUIDs (v1 and v2)
- Generate ULIDs
- Multiple output formats (simple, hyphenated, URN)
- Support for batch generation
- Custom prefix and suffix support
- Banner-free mode for script integration

## Installation

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

Generate without banner:
```bash
idgen -nb
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
idgen -u4 -c 5
```

### API Testing & Development
Generate unique IDs on the fly when testing APIs with `curl` or Postman, especially for endpoints that require a unique `request_id` or `transaction_id`.

```bash
# Use in a curl request
curl -X POST https://api.example.com/users \
  -H "X-Request-ID: $(idgen -u4 -s -nb)" \
  -d '{"name": "John"}'
```

### Scripting & Automation
Use in CI/CD pipelines or shell scripts to generate unique filenames, deployment tags, or temporary resource identifiers.

```bash
# Create a unique temporary file
touch $(idgen -n 10 -p temp_ -f .log -nb)
```

### Configuration & Secrets
Generate unique strings for configuration files, such as `JWT_SECRET`, `API_KEY`, or session secrets during project setup.

```bash
# Generate a strong, random secret
idgen -n 64
```

### Verification & Debugging
Verify the output of deterministic IDs (like UUID v5) to ensure your application logic matches the standard.

```bash
# Verify UUID v5 generation
idgen -u5 --namespace URL --name "https://example.com"
```

### Distributed Tracing
Generate a unique trace ID to manually tag a request flow across microservices when debugging.

```bash
# Generate a trace ID (UUID v4)
idgen -u4 -s
```

### Mock Data Generation
Generate IDs for JSON mock files used in frontend development.

```bash
# Generate 10 IDs for a mock user list
idgen -n 10 -c 10

# Generate mock email addresses
idgen -n 8 -f @example.com -c 5 -nb
```

### Cloud Resource Naming
Generate unique tags for cloud resources (AWS/Azure/GCP) during manual provisioning or Terraform/Ansible runs.

```bash
# Generate a unique suffix for an S3 bucket
idgen -n 8 -p my-bucket- -nb | tr '[:upper:]' '[:lower:]'
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
idgen [OPTIONS]

FLAGS:
    -h --help       Show help information
    -v --version    Show version information
    -nb --no-banner Suppress banner output

UUID VERSION OPTIONS:
    -u1 --uuid1     UUID v1 (Time-based)
    -u3 --uuid3     UUID v3 (MD5 hash-based)
    -u4 --uuid4     UUID v4 (Random - Default)
    -u5 --uuid5     UUID v5 (SHA1 hash-based)

FORMAT OPTIONS:
    -s --simple     Output without hyphens
    -u --urn       Output as URN
    -o --objectid   Generate MongoDB ObjectID
    -d --hyphen     Standard UUID format (Default)
    -n --nanoid     Generate NanoID
    -c1 --cuid1     Generate CUID v1
    -c2 --cuid2     Generate CUID v2
    -l  --ulid      Generate ULID

OTHER OPTIONS:
    -c --count      Number of IDs to generate (Default: 1)
    -p --prefix     Add prefix to generated IDs
    -f --suffix     Add suffix to generated IDs
    --namespace     UUID namespace for v3/v5
    --name         Name string for v3/v5
```

### Format Options
Each ID can be formatted in different ways:

- **Simple**: No separators (`550e8400e29b44d4a716446655440000`)
- **Hyphenated**: Standard format (`550e8400-e29b-44d4-a716-446655440000`)
- **URN**: URN format (`urn:uuid:550e8400-e29b-44d4-a716-446655440000`)

### Examples
```bash
# Various UUID versions
idgen -u4                    # Random UUID v4
idgen -u1                    # Time-based UUID v1
idgen -u5 --namespace 6ba7b810-9dad-11d1-80b4-00c04fd430c8 --name example.com

# Different formats
idgen -s                     # Simple format (no hyphens)
idgen -u                     # URN format
idgen -o                     # MongoDB ObjectID
idgen -n 10                 # NanoID length 10
idgen -c1                    # CUID v1
idgen -c2                    # CUID v2
idgen -l                     # ULID

# Output options
idgen -c 5                  # Generate 5 IDs
idgen -p 'test-' -c 3       # Add prefix
idgen -f '.log'             # Add suffix
idgen -nb                   # No banner output
```

### Common UUID Namespaces
For UUID v3/v5, use these standard namespaces:
- DNS: `6ba7b810-9dad-11d1-80b4-00c04fd430c8`
- URL: `6ba7b811-9dad-11d1-80b4-00c04fd430c8`
- OID: `6ba7b812-9dad-11d1-80b4-00c04fd430c8`
- X500: `6ba7b814-9dad-11d1-80b4-00c04fd430c8`

## Contributing

We welcome contributions! Please feel free to submit a Pull Request. For major changes, open an issue first.

## License

MIT License - Copyright © 2021-2025 [ManiarTech®](https://www.maniartech.com/)