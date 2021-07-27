# uuidgen

Written in RUST this tiny utility (`< 200kb`) quickly generates UUID(v4) and MongoDB ObjectIDs. This library is useful during development and testing when you need to generate UUIDs and Object for your entities.


```sh
USAGE:
    uuidgen [OPTIONS]

FLAGS:
    -h --help       Prints the help information
    -v --version    Prints the version information
    -s --simple     Generates a simple UUID-v4 without hyphen
    -u --urn        Generates the UUID-v4 with URN signature
    -o --objectid   Generates the sequential mongodb ObjectID
    -d --hyphen     Generates the hyphened version of UUID-v4 (Default)

OPTIONS:
    -c --count <num>  Number of times the ids need to be printed (Default 1)
```

Copoyright © 2021 Mohamed Aamir Maniar. All rights reserved.
