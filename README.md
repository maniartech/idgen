# uuidgen

Written in RUST this tiny utility (`< 200kb`) quickly generates UUID(v4) and MongoDB ObjectIDs. Now supports NonoIDs too. This library is useful during development and testing when you need to generate UUIDs and Object for your entities.


```sh
UUID Generator Version 1.1.0
Mohamed Aamir Maniar - https://www.linkedin.com/in/aamironline/
Generates and prints the UUID (or ObjectID) for the specified number of times.

USAGE:
    uuidgen [OPTIONS]

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
```

Copoyright © 2021 ManiarTech. All rights reserved.
