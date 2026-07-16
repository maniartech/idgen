#!/bin/bash
#
# Publish script for idgen
# ========================
#
# Publishes to crates.io and generates package manifests for other platforms.
#
#   ./scripts/publish.sh status        # what is published, checked against reality
#   ./scripts/publish.sh crates        # publish to crates.io
#   ./scripts/publish.sh homebrew      # generate the Homebrew formula
#   ./scripts/publish.sh scoop         # generate the Scoop manifest
#   ./scripts/publish.sh aur           # generate AUR PKGBUILDs
#   ./scripts/publish.sh all           # generate every manifest
#   ./scripts/publish.sh <cmd> --dry-run
#
# IDEMPOTENCY
# -----------
# Safe to re-run. Every command checks the actual state before acting, and
# generating a manifest twice produces a byte-identical file.
#
# Publish state is read from crates.io itself, not from a local ledger. The old
# script tracked published versions in scripts/.publish-history — a gitignored,
# machine-local file. It went wrong in both directions: publish from a second
# machine and the file claims nothing was ever published; delete the file and
# the script tries to re-publish a live version. The registry is the only thing
# that actually knows. `.publish-history` is still appended to as a convenience
# log, but nothing branches on it.
#
# crates.io versions are immutable: a published version can never be replaced,
# only yanked. So there is no --force for crates. If a version is live, the only
# path forward is a new version, and the script says so rather than failing at
# the API with a wall of red.
#
# NAMING
# ------
# Nothing here hardcodes the crate or binary name; see scripts/_common.sh for
# why. The short version: this script used to tell users `cargo install idgen`
# while the crate is `idgen-cli`, and `idgen` is a real, unrelated crate owned
# by someone else — so its output installed the wrong package. Names now derive
# from Cargo.toml, and the release asset list derives from the workflow.
#
# PREREQUISITES
#   crates.io  cargo login (one-time)
#   homebrew   a tap repo, e.g. github.com/<org>/homebrew-tap
#   scoop      a bucket repo
#   aur        an AUR account and SSH key

set -uo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_common.sh"

cd "$REPO_ROOT"

PUBLISH_HISTORY="$REPO_ROOT/scripts/.publish-history"
DIST_DIR="$REPO_ROOT/dist/packages"

# ---------------------------------------------------------------------------
# Arguments
# ---------------------------------------------------------------------------
PLATFORM=""
DRY_RUN=false
FORCE=false

for arg in "$@"; do
    case "$arg" in
        --dry-run) DRY_RUN=true ;;
        --force)   FORCE=true ;;
        --help|-h|help) PLATFORM="help" ;;
        -*) die "Unknown option: $arg (try --help)" ;;
        *)  [ -z "$PLATFORM" ] && PLATFORM="$arg" ;;
    esac
done
PLATFORM="${PLATFORM:-help}"

show_help() {
    sed -n '2,46p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
    echo "Current state:"
    echo "  crate:    $CRATE_NAME"
    echo "  binary:   $BIN_NAME"
    echo "  version:  $VERSION"
    echo "  repo:     $REPO_URL"
    echo ""
}

if [ "$PLATFORM" = "help" ]; then show_help; exit 0; fi

# ---------------------------------------------------------------------------
# Real-world state
# ---------------------------------------------------------------------------

# Is $VERSION of $CRATE_NAME already live on crates.io? Asks the registry.
# Returns 0 (published), 1 (not published), or 2 (could not tell — offline,
# rate-limited, API change). Callers must treat 2 as "unknown" and refuse to
# guess: assuming "not published" would mean attempting a publish that cannot
# be undone.
crates_io_has_version() {
    command -v curl >/dev/null 2>&1 || return 2
    local body
    body="$(curl -fsS --max-time 20 \
        -H "User-Agent: $CRATE_NAME-publish-script" \
        "https://crates.io/api/v1/crates/$CRATE_NAME/versions" 2>/dev/null)" || return 2
    [ -n "$body" ] || return 2
    # Match "num":"1.5.0" tolerating whitespace. Avoids a jq dependency.
    if echo "$body" | grep -qE "\"num\"[[:space:]]*:[[:space:]]*\"$(echo "$VERSION" | sed 's/\./\\./g')\""; then
        return 0
    fi
    # Confirm we actually parsed a version list before claiming "not published".
    echo "$body" | grep -qE '"num"[[:space:]]*:' || return 2
    return 1
}

# Are the release assets for this version actually downloadable yet?
release_assets_available() {
    command -v curl >/dev/null 2>&1 || return 2
    local first code
    first="$(release_assets | head -1)"
    code="$(curl -s -o /dev/null -w "%{http_code}" -IL --max-time 20 "$RELEASE_URL/$first" 2>/dev/null)"
    [ "$code" = "200" ]
}

record_publish() {
    local platform=$1
    $DRY_RUN && return 0
    # A convenience log only. Nothing reads this to make decisions.
    touch "$PUBLISH_HISTORY"
    local entry="${VERSION}:${platform}:$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    grep -q "^${VERSION}:${platform}:" "$PUBLISH_HISTORY" 2>/dev/null || echo "$entry" >> "$PUBLISH_HISTORY"
}

# sha256 of a URL, or empty if it cannot be fetched. Never invents a value:
# a wrong hash in a formula is worse than an obvious placeholder.
sha256_of_url() {
    local url=$1 tmp hash
    command -v curl >/dev/null 2>&1 || return 1
    tmp="$(mktemp 2>/dev/null || echo "/tmp/pub-$$")"
    if curl -fsSL --max-time 120 -o "$tmp" "$url" 2>/dev/null && [ -s "$tmp" ]; then
        if command -v sha256sum >/dev/null 2>&1; then
            hash="$(sha256sum "$tmp" | cut -d' ' -f1)"
        elif command -v shasum >/dev/null 2>&1; then
            hash="$(shasum -a 256 "$tmp" | cut -d' ' -f1)"
        fi
    fi
    rm -f "$tmp"
    [ -n "${hash:-}" ] || return 1
    echo "$hash"
}

write_file() {
    local path=$1 content=$2
    if $DRY_RUN; then
        echo "${GRAY}   would write: $path${NC}"
        return 0
    fi
    mkdir -p "$(dirname "$path")" || die "Cannot create $(dirname "$path")"
    # Idempotent: identical input produces an identical file, and rewriting it
    # is a no-op rather than a spurious change.
    printf '%s\n' "$content" > "$path" || die "Cannot write $path"
    success "Generated: ${path#$REPO_ROOT/}"
}

# ---------------------------------------------------------------------------
# status
# ---------------------------------------------------------------------------
show_status() {
    echo "${BOLD}Status for $CRATE_NAME v$VERSION${NC}"
    echo ""

    crates_io_has_version
    case $? in
        0) echo "  ${GREEN}published${NC}  crates.io  ${GRAY}(v$VERSION is live)${NC}" ;;
        1) echo "  ${RED}pending${NC}    crates.io  ${GRAY}(v$VERSION not published)${NC}" ;;
        *) echo "  ${YELLOW}unknown${NC}    crates.io  ${GRAY}(could not reach the registry)${NC}" ;;
    esac

    local tag="v$VERSION"
    if git rev-parse -q --verify "refs/tags/$tag" >/dev/null 2>&1; then
        echo "  ${GREEN}tagged${NC}     git        ${GRAY}($tag exists locally)${NC}"
    else
        echo "  ${RED}untagged${NC}   git        ${GRAY}($tag not found — run ./scripts/release.sh $VERSION)${NC}"
    fi

    if release_assets_available; then
        echo "  ${GREEN}available${NC}  assets     ${GRAY}(release binaries downloadable)${NC}"
    else
        echo "  ${YELLOW}waiting${NC}    assets     ${GRAY}(not downloadable yet)${NC}"
    fi

    echo ""
    echo "  ${GRAY}Expected assets (from .github/workflows/release.yml):${NC}"
    release_assets | sed "s/^/    ${GRAY}- /;s/$/${NC}/"
    echo ""
    for f in "$DIST_DIR/$BIN_NAME.rb" "$DIST_DIR/$BIN_NAME.json" "$DIST_DIR/aur/PKGBUILD-bin"; do
        [ -f "$f" ] && echo "  ${GRAY}generated: ${f#$REPO_ROOT/}${NC}"
    done
    echo ""
}

# ---------------------------------------------------------------------------
# crates.io
# ---------------------------------------------------------------------------
publish_crates() {
    echo "${BOLD}Publishing $CRATE_NAME v$VERSION to crates.io${NC}"
    echo ""

    command -v cargo >/dev/null 2>&1 || die "cargo not found. Install Rust: https://rustup.rs"

    # Idempotency gate, against the registry rather than a local file.
    crates_io_has_version
    case $? in
        0)
            success "v$VERSION is already on crates.io — nothing to do"
            hint "Versions on crates.io are immutable; they can be yanked but never replaced."
            hint "To ship changes, bump the version and release again."
            record_publish "crates"
            return 0
            ;;
        2)
            # Refuse to guess. Publishing is irreversible.
            die "Could not reach crates.io to check whether v$VERSION is published.
$(hint "Publishing is irreversible, so this script will not guess.")
$(hint "Check $REPO_URL and https://crates.io/crates/$CRATE_NAME, then retry.")"
            ;;
    esac
    info "v$VERSION is not yet on crates.io"

    if [ ! -f "$HOME/.cargo/credentials" ] && [ ! -f "$HOME/.cargo/credentials.toml" ]; then
        die "Not logged in to crates.io.
$(hint "Create a token at https://crates.io/me with publish-new and publish-update scopes")
$(hint "then run: cargo login <token>")"
    fi
    success "crates.io credentials found"

    echo ""
    info "Validating the package (dry run)..."
    cargo publish --dry-run --locked 2>&1 | tail -5 \
        || die "cargo publish --dry-run failed. Fix the errors above before publishing."
    success "Package validates"

    if $DRY_RUN; then
        echo ""
        echo "${BLUE}Dry run — stopping before the irreversible step.${NC}"
        return 0
    fi

    echo ""
    warn "Publishing to crates.io cannot be undone. A version can be yanked, never replaced."
    read -r -p "Publish $CRATE_NAME v$VERSION? (y/N) " -n 1 reply
    echo ""
    if [[ ! "$reply" =~ ^[Yy]$ ]]; then
        info "Cancelled — nothing was published"
        return 0
    fi

    cargo publish --locked || die "cargo publish failed"

    success "Published $CRATE_NAME v$VERSION"
    record_publish "crates"
    echo ""
    echo "  View:    https://crates.io/crates/$CRATE_NAME"
    echo "  Install: cargo install $CRATE_NAME"
    echo ""
}

# ---------------------------------------------------------------------------
# Homebrew
# ---------------------------------------------------------------------------
generate_homebrew() {
    echo "${BOLD}Generating Homebrew formula${NC}"
    echo ""

    release_assets_available || warn "Release assets are not downloadable yet — hashes will be placeholders"

    # Only emit a branch for an asset the workflow actually builds. The old
    # formula hardcoded an arm64 branch pointing at idgen-macos-arm64 while the
    # workflow built no such asset, so every Apple Silicon install failed.
    local mac_arm mac_amd linux
    mac_arm="$(asset_for 'macos-arm64' || true)"
    mac_amd="$(asset_for 'macos-amd64' || true)"
    linux="$(asset_for 'linux-amd64' || true)"

    [ -n "$mac_amd" ] || [ -n "$mac_arm" ] || [ -n "$linux" ] \
        || die "The workflow builds no macOS or Linux asset — nothing to put in a formula."

    if [ -z "$mac_arm" ]; then
        warn "No macos-arm64 asset in the workflow — the formula will omit Apple Silicon."
        hint "Add an aarch64-apple-darwin target to .github/workflows/release.yml"
    fi

    local body="" install="" placeholder=false
    if [ -n "$mac_arm" ] || [ -n "$mac_amd" ]; then
        body+="  on_macos do"$'\n'
        if [ -n "$mac_arm" ] && [ -n "$mac_amd" ]; then
            local h_arm h_amd
            h_arm="$(sha256_of_url "$RELEASE_URL/$mac_arm" || true)"
            h_amd="$(sha256_of_url "$RELEASE_URL/$mac_amd" || true)"
            [ -n "$h_arm" ] || { h_arm="PLACEHOLDER_SHA256_ARM64"; placeholder=true; }
            [ -n "$h_amd" ] || { h_amd="PLACEHOLDER_SHA256_AMD64"; placeholder=true; }
            body+="    if Hardware::CPU.arm?"$'\n'
            body+="      url \"$RELEASE_URL/$mac_arm\""$'\n'
            body+="      sha256 \"$h_arm\""$'\n'
            body+="    else"$'\n'
            body+="      url \"$RELEASE_URL/$mac_amd\""$'\n'
            body+="      sha256 \"$h_amd\""$'\n'
            body+="    end"$'\n'
            install+="    bin.install \"$mac_arm\" => \"$BIN_NAME\" if OS.mac? && Hardware::CPU.arm?"$'\n'
            install+="    bin.install \"$mac_amd\" => \"$BIN_NAME\" if OS.mac? && !Hardware::CPU.arm?"$'\n'
        else
            local only="${mac_arm:-$mac_amd}" h
            h="$(sha256_of_url "$RELEASE_URL/$only" || true)"
            [ -n "$h" ] || { h="PLACEHOLDER_SHA256"; placeholder=true; }
            body+="    url \"$RELEASE_URL/$only\""$'\n'
            body+="    sha256 \"$h\""$'\n'
            install+="    bin.install \"$only\" => \"$BIN_NAME\" if OS.mac?"$'\n'
        fi
        body+="  end"$'\n\n'
    fi
    if [ -n "$linux" ]; then
        local h_linux
        h_linux="$(sha256_of_url "$RELEASE_URL/$linux" || true)"
        [ -n "$h_linux" ] || { h_linux="PLACEHOLDER_SHA256_LINUX"; placeholder=true; }
        body+="  on_linux do"$'\n'
        body+="    url \"$RELEASE_URL/$linux\""$'\n'
        body+="    sha256 \"$h_linux\""$'\n'
        body+="  end"$'\n\n'
        install+="    bin.install \"$linux\" => \"$BIN_NAME\" if OS.linux?"$'\n'
    fi

    local class_name
    # idgen-cli -> IdgenCli, matching Homebrew's file-to-class convention.
    class_name="$(echo "$BIN_NAME" | awk -F'[-_]' '{for(i=1;i<=NF;i++) printf toupper(substr($i,1,1)) tolower(substr($i,2))}')"

    local desc
    desc="$(cargo_field '[package]' 'description')"

    write_file "$DIST_DIR/$BIN_NAME.rb" "# Homebrew formula for $BIN_NAME
# Generated by scripts/publish.sh — do not edit by hand.
#
#   brew tap <org>/tap && brew install $BIN_NAME

class $class_name < Formula
  desc \"${desc%%.*}\"
  homepage \"$REPO_URL\"
  version \"$VERSION\"
  license \"$(cargo_field '[package]' 'license')\"

$body  def install
$install
    generate_completions_from_executable(bin/\"$BIN_NAME\", \"completions\")
  end

  test do
    # Default output must be a UUID v4: the version nibble is 4 and the variant
    # nibble is 8, 9, a or b.
    output = shell_output(\"#{bin}/$BIN_NAME\").strip
    assert_match(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}\$/i, output)

    # A v7 must be sortable, so a burst comes back already ordered.
    ids = shell_output(\"#{bin}/$BIN_NAME -t uuid7 -c 5\").split
    assert_equal ids.sort, ids

    assert_match version.to_s, shell_output(\"#{bin}/$BIN_NAME --version\")
  end
end"

    record_publish "homebrew"
    if $placeholder; then
        echo ""
        warn "The formula contains placeholder hashes — do not ship it as-is."
        hint "Re-run once the release assets are downloadable and they will fill in."
    fi
    echo ""
}

# ---------------------------------------------------------------------------
# Scoop
# ---------------------------------------------------------------------------
generate_scoop() {
    echo "${BOLD}Generating Scoop manifest${NC}"
    echo ""

    local win hash
    win="$(asset_for 'windows' || true)"
    [ -n "$win" ] || die "The workflow builds no Windows asset — nothing to put in a Scoop manifest."

    release_assets_available || warn "Release assets are not downloadable yet — hash will be a placeholder"
    hash="$(sha256_of_url "$RELEASE_URL/$win" || true)"
    local placeholder=false
    [ -n "$hash" ] || { hash="PLACEHOLDER_SHA256"; placeholder=true; }

    write_file "$DIST_DIR/$BIN_NAME.json" "{
    \"##\": \"Generated by scripts/publish.sh — do not edit by hand.\",
    \"version\": \"$VERSION\",
    \"description\": \"$(cargo_field '[package]' 'description')\",
    \"homepage\": \"$REPO_URL\",
    \"license\": \"$(cargo_field '[package]' 'license')\",
    \"architecture\": {
        \"64bit\": {
            \"url\": \"$RELEASE_URL/$win\",
            \"hash\": \"$hash\"
        }
    },
    \"bin\": [[\"$win\", \"$BIN_NAME\"]],
    \"checkver\": {
        \"github\": \"$REPO_URL\"
    },
    \"autoupdate\": {
        \"architecture\": {
            \"64bit\": {
                \"url\": \"$REPO_URL/releases/download/v\$version/$win\"
            }
        }
    }
}"

    record_publish "scoop"
    $placeholder && { echo ""; warn "Manifest contains a placeholder hash — re-run once assets are live."; }
    echo ""
}

# ---------------------------------------------------------------------------
# AUR
# ---------------------------------------------------------------------------
generate_aur() {
    echo "${BOLD}Generating AUR PKGBUILDs${NC}"
    echo ""

    local linux
    linux="$(asset_for 'linux-amd64' || true)"
    [ -n "$linux" ] || die "The workflow builds no Linux asset — nothing to put in a PKGBUILD."

    release_assets_available || warn "Release assets are not downloadable yet — hashes will be placeholders"

    local bin_sha src_sha placeholder=false
    bin_sha="$(sha256_of_url "$RELEASE_URL/$linux" || true)"
    src_sha="$(sha256_of_url "$REPO_URL/archive/v$VERSION.tar.gz" || true)"
    [ -n "$bin_sha" ] || { bin_sha="PLACEHOLDER_SHA256"; placeholder=true; }
    [ -n "$src_sha" ] || { src_sha="PLACEHOLDER_SHA256"; placeholder=true; }

    local desc maint
    desc="$(cargo_field '[package]' 'description')"
    maint="$(cargo_field '[package]' 'authors' || echo 'Maintainer')"

    write_file "$DIST_DIR/aur/PKGBUILD-bin" "# Maintainer: $maint
# Generated by scripts/publish.sh — do not edit by hand.
pkgname=$BIN_NAME-bin
pkgver=$VERSION
pkgrel=1
pkgdesc=\"$desc\"
arch=('x86_64')
url=\"$REPO_URL\"
license=('$(cargo_field '[package]' 'license')')
provides=('$BIN_NAME')
conflicts=('$BIN_NAME')
source=(\"\$url/releases/download/v\\\${pkgver}/$linux\")
sha256sums=('$bin_sha')

package() {
    install -Dm755 \"\\\${srcdir}/$linux\" \"\\\${pkgdir}/usr/bin/$BIN_NAME\"

    # Completions are generated by the binary we just installed.
    \"\\\${pkgdir}/usr/bin/$BIN_NAME\" completions bash > $BIN_NAME.bash
    \"\\\${pkgdir}/usr/bin/$BIN_NAME\" completions zsh  > _$BIN_NAME
    \"\\\${pkgdir}/usr/bin/$BIN_NAME\" completions fish > $BIN_NAME.fish

    install -Dm644 $BIN_NAME.bash \"\\\${pkgdir}/usr/share/bash-completion/completions/$BIN_NAME\"
    install -Dm644 _$BIN_NAME \"\\\${pkgdir}/usr/share/zsh/site-functions/_$BIN_NAME\"
    install -Dm644 $BIN_NAME.fish \"\\\${pkgdir}/usr/share/fish/vendor_completions.d/$BIN_NAME.fish\"
}"

    write_file "$DIST_DIR/aur/PKGBUILD-src" "# Maintainer: $maint
# Generated by scripts/publish.sh — do not edit by hand.
pkgname=$BIN_NAME
pkgver=$VERSION
pkgrel=1
pkgdesc=\"$desc\"
arch=('x86_64')
url=\"$REPO_URL\"
license=('$(cargo_field '[package]' 'license')')
makedepends=('rust' 'cargo')
source=(\"\\\$pkgname-\\\$pkgver.tar.gz::\$url/archive/v\\\${pkgver}.tar.gz\")
sha256sums=('$src_sha')

build() {
    cd \"\\\$pkgname-\\\$pkgver\"
    cargo build --release --locked
}

package() {
    cd \"\\\$pkgname-\\\$pkgver\"
    install -Dm755 \"target/release/$BIN_NAME\" \"\\\${pkgdir}/usr/bin/$BIN_NAME\"

    ./target/release/$BIN_NAME completions bash > $BIN_NAME.bash
    ./target/release/$BIN_NAME completions zsh  > _$BIN_NAME
    ./target/release/$BIN_NAME completions fish > $BIN_NAME.fish

    install -Dm644 $BIN_NAME.bash \"\\\${pkgdir}/usr/share/bash-completion/completions/$BIN_NAME\"
    install -Dm644 _$BIN_NAME \"\\\${pkgdir}/usr/share/zsh/site-functions/_$BIN_NAME\"
    install -Dm644 $BIN_NAME.fish \"\\\${pkgdir}/usr/share/fish/vendor_completions.d/$BIN_NAME.fish\"
    install -Dm644 LICENSE \"\\\${pkgdir}/usr/share/licenses/\\\$pkgname/LICENSE\"
}"

    record_publish "aur"
    $placeholder && { echo ""; warn "PKGBUILDs contain placeholder hashes — re-run once assets are live."; }
    echo ""
    hint "Generate .SRCINFO in the AUR checkout with: makepkg --printsrcinfo > .SRCINFO"
    echo ""
}

# ---------------------------------------------------------------------------
# binstall
# ---------------------------------------------------------------------------
generate_binstall() {
    echo "${BOLD}cargo-binstall metadata${NC}"
    echo ""
    echo "Add to Cargo.toml to enable 'cargo binstall $CRATE_NAME':"
    echo ""
    echo "[package.metadata.binstall]"
    echo "pkg-fmt = \"bin\""
    release_assets | while read -r asset; do
        local target=""
        case "$asset" in
            *windows*)     target="x86_64-pc-windows-msvc" ;;
            *linux-amd64*) target="x86_64-unknown-linux-musl" ;;
            *macos-amd64*) target="x86_64-apple-darwin" ;;
            *macos-arm64*) target="aarch64-apple-darwin" ;;
        esac
        [ -n "$target" ] || continue
        echo ""
        echo "[package.metadata.binstall.overrides.$target]"
        echo "pkg-url = \"{ repo }/releases/download/v{ version }/$asset\""
    done
    echo ""
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
$DRY_RUN && { echo "${BLUE}${BOLD}DRY RUN${NC} ${BLUE}- no files written, nothing published${NC}"; echo ""; }

case "$PLATFORM" in
    crates)   publish_crates ;;
    homebrew) generate_homebrew ;;
    scoop)    generate_scoop ;;
    aur)      generate_aur ;;
    binstall) generate_binstall ;;
    status)   show_status ;;
    all)
        generate_homebrew
        generate_scoop
        generate_aur
        generate_binstall
        ;;
    *)
        die "Unknown command: $PLATFORM
$(hint "Valid: crates, homebrew, scoop, aur, binstall, all, status")"
        ;;
esac
