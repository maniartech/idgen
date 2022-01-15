# idgen

Written in RUST this tiny utility quickly generates UUID(v4), NanoID and MongoDB ObjectIDs. This library is useful during development and testing when you need to generate UUIDs and Object for your entities.

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
    -o --objectid         Generates the sequential mongodb ObjectId
    -d --hyphen           Generates the hyphened version of UUID-v4 (Default)
    -n --nanoid   <num?>  Generates the nanoid with the specified length (Default: 21)
    -c --count    <num>   Number of times the ids need to be printed (Default: 1)
    -p --prefix   <str>   Prefix for the generated ids (Default: None)
```

Copoyright © 2021-2022 ManiarTech. All rights reserved.
