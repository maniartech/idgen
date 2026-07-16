#!/bin/bash
#
# Shared definitions for release.sh and publish.sh
# ================================================
#
# WHY THIS FILE EXISTS
# --------------------
# Both scripts need the same handful of facts: what the crate is called, what
# the binary is called, what version we are on, and which assets a release
# produces. Previously each script hardcoded its own answers, and they drifted:
#
#   - publish.sh told users `cargo install idgen`, but the crate is `idgen-cli`.
#     `idgen` is an unrelated crate owned by someone else, so that output sent
#     users to the wrong package.
#   - publish.sh's Homebrew formula referenced an `idgen-macos-arm64` asset that
#     the release workflow never built, so the formula was broken on Apple
#     Silicon.
#
# Both bugs were the same class of failure: a fact duplicated in two places,
# where changing one did not change the other. So nothing here is hardcoded.
# Every value is derived from the file that actually owns it:
#
#   Cargo.toml                   -> crate name, binary name, version, repo URL
#   .github/workflows/release.yml -> the list of release assets
#
# If you add a build target to the workflow, publish.sh picks it up with no
# edit here. If you rename the crate, every message follows automatically.
#
# USAGE
#   source "$(dirname "${BASH_SOURCE[0]}")/_common.sh"
#
# Sourcing this file sets: REPO_ROOT, CRATE_NAME, BIN_NAME, VERSION, REPO_URL,
# RELEASE_URL. It exits non-zero if any of them cannot be determined, because a
# release script guessing at its own package name is worse than one that stops.

set -uo pipefail

# --- locate the repo root, regardless of where the caller invoked us from ----
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKFLOW_FILE="$REPO_ROOT/.github/workflows/release.yml"
CARGO_TOML="$REPO_ROOT/Cargo.toml"

# --- colours ----------------------------------------------------------------
# Only emit colour to a terminal. Piping to a file or a CI log gets plain text.
# Same principle the CLI itself uses: detect the tty, never sniff for a CI
# variable, since that misses `./scripts/release.sh | tee log` too.
if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
    RED=$'\033[0;31m'; GREEN=$'\033[0;32m'; YELLOW=$'\033[1;33m'
    BLUE=$'\033[0;34m'; CYAN=$'\033[0;36m'; GRAY=$'\033[0;90m'
    BOLD=$'\033[1m';   NC=$'\033[0m'
else
    RED=''; GREEN=''; YELLOW=''; BLUE=''; CYAN=''; GRAY=''; BOLD=''; NC=''
fi

die()     { echo "${RED}${BOLD}ERROR:${NC} ${RED}$*${NC}" >&2; exit 1; }
warn()    { echo "${YELLOW}${BOLD}WARNING:${NC} ${YELLOW}$*${NC}" >&2; }
info()    { echo "${CYAN}i${NC}  $*"; }
success() { echo "${GREEN}+${NC}  $*"; }
hint()    { echo "${GRAY}   -> $*${NC}"; }

# --- derivations ------------------------------------------------------------
# Read a `name = "value"` key from a specific Cargo.toml section. Scoped to the
# section because Cargo.toml has several `name =` keys ([package] and [[bin]]),
# and a naive `grep '^name'` picks whichever comes first — the bug that made
# these scripts confuse the crate name with the binary name.
cargo_field() {
    local section=$1 key=$2
    awk -v section="$section" -v key="$key" '
        /^\[/            { in_section = ($0 == section) }
        in_section && $1 == key {
            # value is the first double-quoted string on the line
            if (match($0, /"[^"]*"/)) {
                print substr($0, RSTART + 1, RLENGTH - 2)
                exit
            }
        }
    ' "$CARGO_TOML"
}

[ -f "$CARGO_TOML" ] || die "Cargo.toml not found at $CARGO_TOML"

# The crate as published to crates.io — [package] name.
CRATE_NAME="$(cargo_field '[package]' 'name')"
# The executable users actually run — [[bin]] name. Falls back to the crate
# name, which is Cargo's own default when [[bin]] is absent.
BIN_NAME="$(cargo_field '[[bin]]' 'name')"
[ -n "$BIN_NAME" ] || BIN_NAME="$CRATE_NAME"
VERSION="$(cargo_field '[package]' 'version')"
REPO_URL="$(cargo_field '[package]' 'repository')"

[ -n "$CRATE_NAME" ] || die "Could not read [package] name from Cargo.toml"
[ -n "$VERSION" ]    || die "Could not read [package] version from Cargo.toml"
[ -n "$REPO_URL" ]   || die "Could not read [package] repository from Cargo.toml"

REPO_URL="${REPO_URL%/}"   # tolerate a trailing slash
RELEASE_URL="$REPO_URL/releases/download/v$VERSION"

# --- release assets ---------------------------------------------------------
# Parsed from the workflow so the two can never disagree. Add a target there and
# it appears here for free.
release_assets() {
    [ -f "$WORKFLOW_FILE" ] || die "Workflow not found at $WORKFLOW_FILE"
    local assets
    assets="$(grep -E '^[[:space:]]*asset_name:' "$WORKFLOW_FILE" \
              | sed 's/.*asset_name:[[:space:]]*//' | tr -d '\r')"
    [ -n "$assets" ] || die "No asset_name entries found in $WORKFLOW_FILE"
    echo "$assets"
}

# Find the single asset matching a pattern, e.g. `asset_for macos-arm64`.
# Returns empty and non-zero if the workflow does not build it, which lets
# callers degrade honestly rather than emit a URL that 404s.
asset_for() {
    local pattern=$1 match
    match="$(release_assets | grep -E "$pattern" | head -1)"
    [ -n "$match" ] || return 1
    echo "$match"
}

# --- semver -----------------------------------------------------------------
# Deliberately strict: MAJOR.MINOR.PATCH with optional -prerelease. A typo like
# "v1.5" or "1.5" should stop the release, not produce a tag nobody can install.
is_semver() {
    [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]
}

# --- test counting ----------------------------------------------------------
# Sums the `test result:` lines across every test binary.
#
# The previous implementation took `| tail -1`, which lands on the doc-tests
# line — always "0 passed" for this crate. It cheerfully reported "All 0 tests
# passed" and let the release proceed. A gate that always reads zero is not a
# gate, so this sums instead and the caller asserts the total is non-zero.
count_passed_tests() {
    awk '/^test result:/ { for (i = 1; i <= NF; i++) if ($(i+1) == "passed;") total += $i }
         END { print total + 0 }'
}
