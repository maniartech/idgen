#!/bin/bash
#
# Publish script for idgen
# ========================
#
# This script helps publish idgen to various package registries and platforms.
# It provides options to publish to individual platforms or all at once.
#
# Features:
#   - Idempotent: Safe to run multiple times without side effects
#   - Tracks published versions in scripts/.publish-history
#   - Skips already published version/platform combinations
#
# Usage:
#   ./scripts/publish.sh [platform] [options]
#
# Platforms:
#   crates      Publish to crates.io (Rust package registry)
#   homebrew    Generate Homebrew formula
#   scoop       Generate Scoop manifest (Windows)
#   aur         Generate AUR PKGBUILD (Arch Linux)
#   all         Generate all platform files
#   status      Show publish status for current version
#   help        Show this help message
#
# Options:
#   --force     Force publish even if already published
#
# Examples:
#   ./scripts/publish.sh crates        # Publish to crates.io
#   ./scripts/publish.sh homebrew      # Generate Homebrew formula
#   ./scripts/publish.sh all           # Generate all platform files
#   ./scripts/publish.sh status        # Check what's been published
#   ./scripts/publish.sh crates --force  # Force re-publish
#
# Prerequisites:
#   - For crates.io: cargo login (one-time setup)
#   - For Homebrew: Fork homebrew-core or create your own tap
#   - For AUR: AUR account and SSH key setup
#

# Don't exit on error - we handle errors ourselves
set +e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
GRAY='\033[0;90m'
BOLD='\033[1m'
NC='\033[0m'

# Script directory (for publish history file)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PUBLISH_HISTORY="$SCRIPT_DIR/.publish-history"

# ============================================================================
# ERROR HANDLING & REPORTING
# ============================================================================

# Track errors for summary
ERRORS=()
WARNINGS=()

# Print an error message with formatting
error() {
    echo -e "${RED}${BOLD}ERROR:${NC} ${RED}$1${NC}" >&2
    ERRORS+=("$1")
}

# Print a warning message
warn() {
    echo -e "${YELLOW}${BOLD}WARNING:${NC} ${YELLOW}$1${NC}"
    WARNINGS+=("$1")
}

# Print an info message
info() {
    echo -e "${CYAN}ℹ${NC}  $1"
}

# Print a success message
success() {
    echo -e "${GREEN}✓${NC}  $1"
}

# Print a hint/suggestion
hint() {
    echo -e "${GRAY}   💡 $1${NC}"
}

# Print a command suggestion
suggest_cmd() {
    echo -e "${GRAY}   → ${CYAN}$1${NC}"
}

# Show a troubleshooting guide
troubleshoot() {
    local issue=$1
    echo ""
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}${BOLD}Troubleshooting: $issue${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# Show summary at the end
show_summary() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}Summary${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    if [ ${#ERRORS[@]} -eq 0 ] && [ ${#WARNINGS[@]} -eq 0 ]; then
        echo -e "${GREEN}✓ All operations completed successfully!${NC}"
    else
        if [ ${#WARNINGS[@]} -gt 0 ]; then
            echo -e "${YELLOW}Warnings (${#WARNINGS[@]}):${NC}"
            for warning in "${WARNINGS[@]}"; do
                echo -e "  ${YELLOW}⚠${NC}  $warning"
            done
        fi

        if [ ${#ERRORS[@]} -gt 0 ]; then
            echo -e "${RED}Errors (${#ERRORS[@]}):${NC}"
            for err in "${ERRORS[@]}"; do
                echo -e "  ${RED}✗${NC}  $err"
            done
            echo ""
            echo -e "${YELLOW}Run with --help for usage information${NC}"
        fi
    fi
    echo ""
}

# ============================================================================
# PREREQUISITES VALIDATION
# ============================================================================

# Check if running from project root
validate_project_root() {
    if [ ! -f "Cargo.toml" ]; then
        error "Cargo.toml not found. Please run this script from the project root."
        troubleshoot "Wrong directory"
        echo "  This script must be run from the idgen project root directory."
        echo ""
        echo "  Your current directory: $(pwd)"
        echo ""
        suggest_cmd "cd /path/to/idgen"
        suggest_cmd "./scripts/publish.sh $*"
        echo ""
        exit 1
    fi
}

# Validate version can be extracted
validate_version() {
    if [ -z "$VERSION" ] || [ "$VERSION" == "" ]; then
        error "Could not extract version from Cargo.toml"
        troubleshoot "Version extraction failed"
        echo "  The script expects a line like: version = \"1.4.0\" in Cargo.toml"
        echo ""
        echo "  Check your Cargo.toml format:"
        suggest_cmd "grep version Cargo.toml"
        echo ""
        exit 1
    fi
}

# Check if a command exists
require_command() {
    local cmd=$1
    local install_hint=$2

    if ! command -v "$cmd" &> /dev/null; then
        error "$cmd is not installed or not in PATH"
        if [ -n "$install_hint" ]; then
            troubleshoot "$cmd not found"
            echo "  $install_hint"
            echo ""
        fi
        return 1
    fi
    return 0
}

# Check if git release tag exists
validate_release_exists() {
    local tag="v$VERSION"

    info "Checking if release $tag exists on GitHub..."

    # Check if tag exists locally
    if ! git rev-parse "$tag" &>/dev/null; then
        warn "Git tag $tag not found locally"
        hint "Create a release first: ./scripts/release.sh $VERSION"
    fi

    # Check if release assets are available (using curl)
    if require_command "curl" ""; then
        local check_url="$RELEASE_URL/idgen-linux-amd64"
        local http_code=$(curl -s -o /dev/null -w "%{http_code}" -I "$check_url" 2>/dev/null)

        if [ "$http_code" != "200" ] && [ "$http_code" != "302" ]; then
            warn "Release assets may not be available yet (HTTP $http_code)"
            hint "Wait for GitHub Actions to complete building release assets"
            hint "Check: $REPO_URL/releases/tag/$tag"
            return 1
        else
            success "Release assets are available"
        fi
    fi
    return 0
}

# Run all prerequisite checks
validate_prerequisites() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}Validating prerequisites...${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    validate_project_root
    validate_version

    success "Project root: $(pwd)"
    success "Version: $VERSION"
    echo ""
}

# ============================================================================
# INITIALIZATION
# ============================================================================

# Run validation before extracting version
if [ ! -f "Cargo.toml" ] && [ "${1:-}" != "help" ] && [ "${1:-}" != "--help" ] && [ "${1:-}" != "-h" ]; then
    echo -e "${RED}${BOLD}ERROR:${NC} ${RED}Cargo.toml not found. Please run from project root.${NC}" >&2
    exit 1
fi

# Get version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml 2>/dev/null | head -1 | sed 's/.*"\(.*\)".*/\1/' || echo "")
REPO_URL="https://github.com/maniartech/idgen"
RELEASE_URL="$REPO_URL/releases/download/v$VERSION"

# Create dist directory for generated files
DIST_DIR="dist/packages"
mkdir -p "$DIST_DIR"

# Check for --force flag
FORCE=false
for arg in "$@"; do
    if [ "$arg" == "--force" ]; then
        FORCE=true
    fi
done

# ============================================================================
# PUBLISH HISTORY FUNCTIONS
# ============================================================================

# Check if a version/platform combination has been published
is_published() {
    local platform=$1
    local version=$2

    if [ ! -f "$PUBLISH_HISTORY" ]; then
        return 1  # Not published (file doesn't exist)
    fi

    grep -q "^${version}:${platform}:" "$PUBLISH_HISTORY" 2>/dev/null
}

# Record a successful publish
record_publish() {
    local platform=$1
    local version=$2
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    # Create file if it doesn't exist
    touch "$PUBLISH_HISTORY"

    # Add entry
    echo "${version}:${platform}:${timestamp}" >> "$PUBLISH_HISTORY"

    echo -e "${GRAY}  Recorded in .publish-history${NC}"
}

# Check if should skip (already published and not forced)
should_skip() {
    local platform=$1

    if [ "$FORCE" = true ]; then
        return 1  # Don't skip if forced
    fi

    if is_published "$platform" "$VERSION"; then
        echo -e "${YELLOW}⏭  Skipping $platform v$VERSION (already published)${NC}"
        echo -e "${GRAY}   Use --force to publish again${NC}"
        echo ""
        return 0  # Skip
    fi

    return 1  # Don't skip
}

# Show publish status
show_status() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}Publish Status for v$VERSION${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    local platforms=("crates" "homebrew" "scoop" "aur")
    local not_published=()

    for platform in "${platforms[@]}"; do
        if is_published "$platform" "$VERSION"; then
            local timestamp=$(grep "^${VERSION}:${platform}:" "$PUBLISH_HISTORY" 2>/dev/null | tail -1 | cut -d: -f3-)
            echo -e "  ${GREEN}✓${NC} $platform ${GRAY}(published: $timestamp)${NC}"
        else
            echo -e "  ${RED}✗${NC} $platform ${GRAY}(not published)${NC}"
            not_published+=("$platform")
        fi
    done

    echo ""

    # Show next steps if something is not published
    if [ ${#not_published[@]} -gt 0 ]; then
        echo -e "${YELLOW}To publish remaining platforms:${NC}"
        for platform in "${not_published[@]}"; do
            suggest_cmd "./scripts/publish.sh $platform"
        done
        echo ""
    else
        echo -e "${GREEN}All platforms published for v$VERSION!${NC}"
        echo ""
    fi

    if [ -f "$PUBLISH_HISTORY" ]; then
        echo -e "${GRAY}History file: $PUBLISH_HISTORY${NC}"
        echo -e "${GRAY}Total entries: $(wc -l < "$PUBLISH_HISTORY")${NC}"
        echo ""

        # Show recent history
        local recent_count=$(tail -5 "$PUBLISH_HISTORY" | wc -l)
        if [ $recent_count -gt 0 ]; then
            echo -e "${GRAY}Recent publishes:${NC}"
            tail -5 "$PUBLISH_HISTORY" | while read line; do
                echo -e "  ${GRAY}$line${NC}"
            done
        fi
    else
        echo -e "${GRAY}No publish history found.${NC}"
        hint "Run './scripts/publish.sh crates' to publish to crates.io"
    fi
    echo ""
}

show_help() {
    echo ""
    echo -e "${BOLD}idgen publish script${NC}"
    echo ""
    echo -e "${YELLOW}Usage:${NC} ./scripts/publish.sh [platform] [options]"
    echo ""
    echo -e "${YELLOW}Platforms:${NC}"
    echo "  crates      Publish to crates.io (Rust package registry)"
    echo "  homebrew    Generate Homebrew formula"
    echo "  scoop       Generate Scoop manifest (Windows)"
    echo "  aur         Generate AUR PKGBUILD (Arch Linux)"
    echo "  all         Generate all platform files"
    echo "  status      Show publish status for current version"
    echo "  help        Show this help message"
    echo ""
    echo -e "${YELLOW}Options:${NC}"
    echo "  --force     Force publish even if already published"
    echo "  --check     Validate prerequisites without publishing"
    echo ""
    echo -e "${YELLOW}Examples:${NC}"
    echo "  ./scripts/publish.sh status          # Check what's been published"
    echo "  ./scripts/publish.sh crates          # Publish to crates.io"
    echo "  ./scripts/publish.sh all             # Generate all platform files"
    echo "  ./scripts/publish.sh crates --force  # Force re-publish"
    echo "  ./scripts/publish.sh --check         # Validate prerequisites"
    echo ""
    echo -e "${YELLOW}Current State:${NC}"
    echo "  Version:      ${VERSION:-unknown}"
    echo "  History file: $PUBLISH_HISTORY"
    echo ""
    echo -e "${YELLOW}Workflow:${NC}"
    echo "  1. Create release:  ./scripts/release.sh 1.4.0"
    echo "  2. Wait for GitHub Actions to build release assets"
    echo "  3. Publish:         ./scripts/publish.sh crates"
    echo "  4. Generate files:  ./scripts/publish.sh all"
    echo ""
}

# ============================================================================
# CRATES.IO
# ============================================================================
publish_crates() {
    if should_skip "crates"; then
        return 0
    fi

    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}Publishing to crates.io...${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    # Check cargo is available
    if ! require_command "cargo" "Install Rust: https://rustup.rs"; then
        return 1
    fi
    success "cargo found"

    # Check if logged in to crates.io
    info "Checking crates.io authentication..."
    if ! cargo login --help &>/dev/null; then
        error "cargo login command not available"
        return 1
    fi

    # Check if credentials exist
    local cargo_credentials="$HOME/.cargo/credentials"
    local cargo_credentials_toml="$HOME/.cargo/credentials.toml"

    if [ ! -f "$cargo_credentials" ] && [ ! -f "$cargo_credentials_toml" ]; then
        error "Not logged in to crates.io"
        troubleshoot "crates.io authentication"
        echo "  You need to authenticate with crates.io before publishing."
        echo ""
        echo "  Steps:"
        echo "  1. Go to https://crates.io/me"
        echo "  2. Create an API token with 'publish-new' and 'publish-update' scopes"
        echo "  3. Run:"
        suggest_cmd "cargo login <your-api-token>"
        echo ""
        return 1
    fi
    success "crates.io credentials found"

    # Dry run first
    echo ""
    info "Running dry-run to validate package..."
    echo ""
    if ! cargo publish --dry-run 2>&1; then
        local exit_code=$?
        error "Dry run failed (exit code: $exit_code)"
        troubleshoot "cargo publish dry-run failed"
        echo "  Common issues:"
        echo "  • Missing required fields in Cargo.toml (description, license, repository)"
        echo "  • Invalid package name or version"
        echo "  • Build errors in the code"
        echo "  • Missing dependencies"
        echo ""
        echo "  Check your Cargo.toml has these fields:"
        suggest_cmd "grep -E '^(name|version|description|license|repository)' Cargo.toml"
        echo ""
        echo "  Try building first:"
        suggest_cmd "cargo build --release"
        echo ""
        return 1
    fi

    echo ""
    success "Dry run passed"
    echo ""

    # Prompt for confirmation
    echo -e "${YELLOW}Ready to publish idgen v$VERSION to crates.io${NC}"
    echo ""
    read -p "Proceed with publishing? (y/N) " -n 1 -r
    echo ""

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        warn "Publishing cancelled by user"
        return 0
    fi

    # Actually publish
    echo ""
    info "Publishing to crates.io..."
    echo ""

    if ! cargo publish 2>&1; then
        local exit_code=$?
        error "Failed to publish to crates.io (exit code: $exit_code)"
        troubleshoot "cargo publish failed"
        echo "  Possible reasons:"
        echo "  • Version $VERSION already exists on crates.io"
        echo "  • Network connectivity issues"
        echo "  • API token expired or invalid"
        echo "  • Rate limiting"
        echo ""
        echo "  Check if version exists:"
        suggest_cmd "curl -s https://crates.io/api/v1/crates/idgen/versions | grep $VERSION"
        echo ""
        echo "  If version exists, bump version in Cargo.toml and try again."
        echo ""
        return 1
    fi

    echo ""
    success "Published to crates.io!"
    record_publish "crates" "$VERSION"
    echo ""
    echo -e "  ${GRAY}View at:${NC} https://crates.io/crates/idgen"
    echo ""
    echo -e "${YELLOW}Users can now install with:${NC}"
    suggest_cmd "cargo install idgen"
    echo ""
}

# ============================================================================
# HOMEBREW
# ============================================================================
generate_homebrew() {
    if should_skip "homebrew"; then
        return 0
    fi

    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}Generating Homebrew formula...${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    # Check release assets are available
    validate_release_exists || warn "Proceeding anyway, but SHA256 hashes will need manual update"

    # Ensure dist directory exists
    if ! mkdir -p "$DIST_DIR" 2>/dev/null; then
        error "Failed to create directory: $DIST_DIR"
        troubleshoot "Directory creation failed"
        echo "  Check write permissions for: $(dirname "$DIST_DIR")"
        suggest_cmd "ls -la $(dirname "$DIST_DIR")"
        return 1
    fi

    # Calculate SHA256 hashes if binaries are available
    local macos_amd64_sha="PLACEHOLDER_AMD64_SHA256"
    local macos_arm64_sha="PLACEHOLDER_ARM64_SHA256"
    local linux_sha="PLACEHOLDER_LINUX_SHA256"

    info "Downloading binaries to calculate SHA256 hashes..."

    if require_command "curl" "Install curl to auto-calculate SHA256 hashes"; then
        # Try to download and hash each binary
        local temp_dir=$(mktemp -d 2>/dev/null || echo "/tmp/idgen-$$")
        mkdir -p "$temp_dir"

        # macOS AMD64
        if curl -sL -o "$temp_dir/macos-amd64" "$RELEASE_URL/idgen-macos-amd64" 2>/dev/null && [ -s "$temp_dir/macos-amd64" ]; then
            macos_amd64_sha=$(sha256sum "$temp_dir/macos-amd64" 2>/dev/null | cut -d' ' -f1 || shasum -a 256 "$temp_dir/macos-amd64" 2>/dev/null | cut -d' ' -f1)
            if [ -n "$macos_amd64_sha" ] && [ "$macos_amd64_sha" != "" ]; then
                success "macOS AMD64 SHA256: ${macos_amd64_sha:0:16}..."
            fi
        else
            warn "Could not download macOS AMD64 binary"
        fi

        # macOS ARM64
        if curl -sL -o "$temp_dir/macos-arm64" "$RELEASE_URL/idgen-macos-arm64" 2>/dev/null && [ -s "$temp_dir/macos-arm64" ]; then
            macos_arm64_sha=$(sha256sum "$temp_dir/macos-arm64" 2>/dev/null | cut -d' ' -f1 || shasum -a 256 "$temp_dir/macos-arm64" 2>/dev/null | cut -d' ' -f1)
            if [ -n "$macos_arm64_sha" ] && [ "$macos_arm64_sha" != "" ]; then
                success "macOS ARM64 SHA256: ${macos_arm64_sha:0:16}..."
            fi
        else
            warn "Could not download macOS ARM64 binary"
        fi

        # Linux AMD64
        if curl -sL -o "$temp_dir/linux-amd64" "$RELEASE_URL/idgen-linux-amd64" 2>/dev/null && [ -s "$temp_dir/linux-amd64" ]; then
            linux_sha=$(sha256sum "$temp_dir/linux-amd64" 2>/dev/null | cut -d' ' -f1 || shasum -a 256 "$temp_dir/linux-amd64" 2>/dev/null | cut -d' ' -f1)
            if [ -n "$linux_sha" ] && [ "$linux_sha" != "" ]; then
                success "Linux AMD64 SHA256: ${linux_sha:0:16}..."
            fi
        else
            warn "Could not download Linux AMD64 binary"
        fi

        # Cleanup
        rm -rf "$temp_dir"
    else
        warn "curl not available - SHA256 hashes will be placeholders"
    fi

    echo ""

    cat > "$DIST_DIR/idgen.rb" << EOF
# Homebrew formula for idgen
# To install: brew install maniartech/tap/idgen
# Or add tap: brew tap maniartech/tap && brew install idgen

class Idgen < Formula
  desc "Fast CLI tool for generating and inspecting unique IDs (UUID, NanoID, CUID, ULID, ObjectID)"
  homepage "$REPO_URL"
  version "$VERSION"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      # Apple Silicon (M1/M2)
      url "$RELEASE_URL/idgen-macos-arm64"
      sha256 "$macos_arm64_sha"
    else
      # Intel Mac
      url "$RELEASE_URL/idgen-macos-amd64"
      sha256 "$macos_amd64_sha"
    end
  end

  on_linux do
    url "$RELEASE_URL/idgen-linux-amd64"
    sha256 "$linux_sha"
  end

  def install
    bin.install "idgen-macos-amd64" => "idgen" if OS.mac? && !Hardware::CPU.arm?
    bin.install "idgen-macos-arm64" => "idgen" if OS.mac? && Hardware::CPU.arm?
    bin.install "idgen-linux-amd64" => "idgen" if OS.linux?

    # Generate and install shell completions
    generate_completions_from_executable(bin/"idgen", "completions")
  end

  test do
    # Test basic UUID generation
    output = shell_output("#{bin}/idgen")
    assert_match(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i, output.strip)

    # Test version
    assert_match version.to_s, shell_output("#{bin}/idgen --version")
  end
end
EOF

    if [ ! -f "$DIST_DIR/idgen.rb" ]; then
        error "Failed to create Homebrew formula"
        return 1
    fi

    success "Generated: $DIST_DIR/idgen.rb"
    record_publish "homebrew" "$VERSION"
    echo ""

    # Check for placeholder hashes and warn
    if grep -q "PLACEHOLDER" "$DIST_DIR/idgen.rb"; then
        warn "Formula contains placeholder SHA256 hashes"
        echo ""
        echo -e "${YELLOW}To update hashes manually:${NC}"
        suggest_cmd "curl -sL $RELEASE_URL/idgen-macos-amd64 | shasum -a 256"
        suggest_cmd "curl -sL $RELEASE_URL/idgen-macos-arm64 | shasum -a 256"
        suggest_cmd "curl -sL $RELEASE_URL/idgen-linux-amd64 | shasum -a 256"
        echo ""
    fi

    echo -e "${YELLOW}Next steps for Homebrew:${NC}"
    echo "  1. Create a tap repository: github.com/maniartech/homebrew-tap"
    echo "  2. Copy idgen.rb to Formula/idgen.rb in the tap"
    echo "  3. Users install with:"
    suggest_cmd "brew tap maniartech/tap"
    suggest_cmd "brew install idgen"
    echo ""
}

# ============================================================================
# SCOOP (Windows)
# ============================================================================
generate_scoop() {
    if should_skip "scoop"; then
        return 0
    fi

    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}Generating Scoop manifest...${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    # Check release assets
    validate_release_exists || warn "Proceeding anyway, but SHA256 hash will need manual update"

    # Ensure dist directory exists
    if ! mkdir -p "$DIST_DIR" 2>/dev/null; then
        error "Failed to create directory: $DIST_DIR"
        return 1
    fi

    # Calculate SHA256 hash
    local windows_sha="PLACEHOLDER_SHA256"

    if require_command "curl" ""; then
        info "Downloading Windows binary to calculate SHA256..."
        local temp_file=$(mktemp 2>/dev/null || echo "/tmp/idgen-win-$$")

        if curl -sL -o "$temp_file" "$RELEASE_URL/idgen-windows-amd64.exe" 2>/dev/null && [ -s "$temp_file" ]; then
            windows_sha=$(sha256sum "$temp_file" 2>/dev/null | cut -d' ' -f1 || shasum -a 256 "$temp_file" 2>/dev/null | cut -d' ' -f1)
            if [ -n "$windows_sha" ] && [ "$windows_sha" != "" ]; then
                success "Windows SHA256: ${windows_sha:0:16}..."
            fi
        else
            warn "Could not download Windows binary"
        fi
        rm -f "$temp_file"
    fi

    echo ""

    cat > "$DIST_DIR/idgen.json" << EOF
{
    "version": "$VERSION",
    "description": "Fast CLI tool for generating and inspecting unique IDs (UUID, NanoID, CUID, ULID, ObjectID)",
    "homepage": "$REPO_URL",
    "license": "MIT",
    "architecture": {
        "64bit": {
            "url": "$RELEASE_URL/idgen-windows-amd64.exe",
            "hash": "$windows_sha"
        }
    },
    "bin": [["idgen-windows-amd64.exe", "idgen"]],
    "checkver": {
        "github": "$REPO_URL"
    },
    "autoupdate": {
        "architecture": {
            "64bit": {
                "url": "$REPO_URL/releases/download/v\$version/idgen-windows-amd64.exe"
            }
        }
    }
}
EOF

    if [ ! -f "$DIST_DIR/idgen.json" ]; then
        error "Failed to create Scoop manifest"
        return 1
    fi

    success "Generated: $DIST_DIR/idgen.json"
    record_publish "scoop" "$VERSION"
    echo ""

    # Check for placeholder and warn
    if grep -q "PLACEHOLDER" "$DIST_DIR/idgen.json"; then
        warn "Manifest contains placeholder SHA256 hash"
        echo ""
        echo -e "${YELLOW}To update hash manually (on Windows):${NC}"
        suggest_cmd "certutil -hashfile idgen-windows-amd64.exe SHA256"
        echo ""
        echo -e "${YELLOW}Or with curl:${NC}"
        suggest_cmd "curl -sL $RELEASE_URL/idgen-windows-amd64.exe | sha256sum"
        echo ""
    fi

    echo -e "${YELLOW}Next steps for Scoop:${NC}"
    echo "  1. Create a bucket repository: github.com/maniartech/scoop-bucket"
    echo "  2. Copy idgen.json to the bucket root"
    echo "  3. Windows users install with:"
    suggest_cmd "scoop bucket add maniartech https://github.com/maniartech/scoop-bucket"
    suggest_cmd "scoop install idgen"
    echo ""
}

# ============================================================================
# AUR (Arch Linux)
# ============================================================================
generate_aur() {
    if should_skip "aur"; then
        return 0
    fi

    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}Generating AUR PKGBUILD...${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    # Check release assets
    validate_release_exists || warn "Proceeding anyway, but SHA256 hashes will need manual update"

    # Ensure dist directory exists
    if ! mkdir -p "$DIST_DIR/aur" 2>/dev/null; then
        error "Failed to create directory: $DIST_DIR/aur"
        return 1
    fi

    # Calculate SHA256 hash
    local linux_sha="PLACEHOLDER_SHA256"
    local source_sha="PLACEHOLDER_SHA256"

    if require_command "curl" ""; then
        info "Downloading Linux binary to calculate SHA256..."
        local temp_file=$(mktemp 2>/dev/null || echo "/tmp/idgen-linux-$$")

        if curl -sL -o "$temp_file" "$RELEASE_URL/idgen-linux-amd64" 2>/dev/null && [ -s "$temp_file" ]; then
            linux_sha=$(sha256sum "$temp_file" 2>/dev/null | cut -d' ' -f1 || shasum -a 256 "$temp_file" 2>/dev/null | cut -d' ' -f1)
            if [ -n "$linux_sha" ] && [ "$linux_sha" != "" ]; then
                success "Linux binary SHA256: ${linux_sha:0:16}..."
            fi
        else
            warn "Could not download Linux binary"
        fi
        rm -f "$temp_file"

        # Try to get source tarball hash
        info "Downloading source tarball to calculate SHA256..."
        local source_url="$REPO_URL/archive/v$VERSION.tar.gz"
        if curl -sL -o "$temp_file" "$source_url" 2>/dev/null && [ -s "$temp_file" ]; then
            source_sha=$(sha256sum "$temp_file" 2>/dev/null | cut -d' ' -f1 || shasum -a 256 "$temp_file" 2>/dev/null | cut -d' ' -f1)
            if [ -n "$source_sha" ] && [ "$source_sha" != "" ]; then
                success "Source tarball SHA256: ${source_sha:0:16}..."
            fi
        else
            warn "Could not download source tarball"
        fi
        rm -f "$temp_file"
    fi

    echo ""

    # Binary package (idgen-bin)
    cat > "$DIST_DIR/aur/PKGBUILD-bin" << EOF
# Maintainer: ManiarTech <contact@maniartech.com>
pkgname=idgen-bin
pkgver=$VERSION
pkgrel=1
pkgdesc="Fast CLI tool for generating and inspecting unique IDs (UUID, NanoID, CUID, ULID, ObjectID)"
arch=('x86_64')
url="https://github.com/maniartech/idgen"
license=('MIT')
provides=('idgen')
conflicts=('idgen')
source=("\$url/releases/download/v\${pkgver}/idgen-linux-amd64")
sha256sums=('$linux_sha')

package() {
    install -Dm755 "\${srcdir}/idgen-linux-amd64" "\${pkgdir}/usr/bin/idgen"

    # Generate shell completions
    "\${pkgdir}/usr/bin/idgen" completions bash > idgen.bash
    "\${pkgdir}/usr/bin/idgen" completions zsh > _idgen
    "\${pkgdir}/usr/bin/idgen" completions fish > idgen.fish

    install -Dm644 idgen.bash "\${pkgdir}/usr/share/bash-completion/completions/idgen"
    install -Dm644 _idgen "\${pkgdir}/usr/share/zsh/site-functions/_idgen"
    install -Dm644 idgen.fish "\${pkgdir}/usr/share/fish/vendor_completions.d/idgen.fish"
}
EOF

    # Source package (idgen)
    cat > "$DIST_DIR/aur/PKGBUILD-src" << EOF
# Maintainer: ManiarTech <contact@maniartech.com>
pkgname=idgen
pkgver=$VERSION
pkgrel=1
pkgdesc="Fast CLI tool for generating and inspecting unique IDs (UUID, NanoID, CUID, ULID, ObjectID)"
arch=('x86_64')
url="https://github.com/maniartech/idgen"
license=('MIT')
makedepends=('rust' 'cargo')
source=("\$pkgname-\$pkgver.tar.gz::\$url/archive/v\${pkgver}.tar.gz")
sha256sums=('$source_sha')

build() {
    cd "\$pkgname-\$pkgver"
    cargo build --release --locked
}

package() {
    cd "\$pkgname-\$pkgver"
    install -Dm755 "target/release/idgen" "\${pkgdir}/usr/bin/idgen"

    # Generate shell completions
    "./target/release/idgen" completions bash > idgen.bash
    "./target/release/idgen" completions zsh > _idgen
    "./target/release/idgen" completions fish > idgen.fish

    install -Dm644 idgen.bash "\${pkgdir}/usr/share/bash-completion/completions/idgen"
    install -Dm644 _idgen "\${pkgdir}/usr/share/zsh/site-functions/_idgen"
    install -Dm644 idgen.fish "\${pkgdir}/usr/share/fish/vendor_completions.d/idgen.fish"
    install -Dm644 LICENSE "\${pkgdir}/usr/share/licenses/\$pkgname/LICENSE"
}
EOF

    # .SRCINFO for binary package
    cat > "$DIST_DIR/aur/.SRCINFO-bin" << EOF
pkgbase = idgen-bin
	pkgdesc = Fast CLI tool for generating and inspecting unique IDs (UUID, NanoID, CUID, ULID, ObjectID)
	pkgver = $VERSION
	pkgrel = 1
	url = https://github.com/maniartech/idgen
	arch = x86_64
	license = MIT
	provides = idgen
	conflicts = idgen
	source = https://github.com/maniartech/idgen/releases/download/v$VERSION/idgen-linux-amd64
	sha256sums = $linux_sha

pkgname = idgen-bin
EOF

    # Check for errors
    local files_created=0
    [ -f "$DIST_DIR/aur/PKGBUILD-bin" ] && files_created=$((files_created + 1))
    [ -f "$DIST_DIR/aur/PKGBUILD-src" ] && files_created=$((files_created + 1))
    [ -f "$DIST_DIR/aur/.SRCINFO-bin" ] && files_created=$((files_created + 1))

    if [ $files_created -ne 3 ]; then
        error "Failed to create some AUR files (created $files_created/3)"
        return 1
    fi

    success "Generated: $DIST_DIR/aur/PKGBUILD-bin"
    success "Generated: $DIST_DIR/aur/PKGBUILD-src"
    success "Generated: $DIST_DIR/aur/.SRCINFO-bin"
    record_publish "aur" "$VERSION"
    echo ""

    # Check for placeholder hashes
    if grep -q "PLACEHOLDER" "$DIST_DIR/aur/PKGBUILD-bin"; then
        warn "PKGBUILD files contain placeholder SHA256 hashes"
        echo ""
        echo -e "${YELLOW}To update hashes:${NC}"
        suggest_cmd "curl -sL $RELEASE_URL/idgen-linux-amd64 | sha256sum"
        suggest_cmd "curl -sL $REPO_URL/archive/v$VERSION.tar.gz | sha256sum"
        echo ""
    fi

    echo -e "${YELLOW}Next steps for AUR:${NC}"
    echo "  1. Create AUR account at: https://aur.archlinux.org/register"
    echo "  2. Set up SSH key for AUR"
    echo "  3. Clone empty package:"
    suggest_cmd "git clone ssh://aur@aur.archlinux.org/idgen-bin.git"
    echo "  4. Copy PKGBUILD-bin as PKGBUILD"
    echo "  5. Generate .SRCINFO:"
    suggest_cmd "makepkg --printsrcinfo > .SRCINFO"
    echo "  6. Commit and push:"
    suggest_cmd "git add PKGBUILD .SRCINFO && git commit -m 'Initial upload' && git push"
    echo "  7. Users install with:"
    suggest_cmd "yay -S idgen-bin"
    echo ""
}

# ============================================================================
# CARGO BINSTALL METADATA
# ============================================================================
generate_binstall() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}Generating cargo-binstall metadata...${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    echo -e "${YELLOW}Add this to Cargo.toml [package.metadata.binstall]:${NC}"
    echo ""
    cat << 'EOF'
[package.metadata.binstall]
pkg-url = "{ repo }/releases/download/v{ version }/idgen-{ target }{ archive-suffix }"
pkg-fmt = "bin"

[package.metadata.binstall.overrides.x86_64-pc-windows-msvc]
pkg-url = "{ repo }/releases/download/v{ version }/idgen-windows-amd64.exe"

[package.metadata.binstall.overrides.x86_64-unknown-linux-musl]
pkg-url = "{ repo }/releases/download/v{ version }/idgen-linux-amd64"

[package.metadata.binstall.overrides.x86_64-apple-darwin]
pkg-url = "{ repo }/releases/download/v{ version }/idgen-macos-amd64"
EOF
    echo ""
    echo -e "${GREEN}This enables: cargo binstall idgen${NC}"
    echo ""
}

# ============================================================================
# GENERATE ALL
# ============================================================================
generate_all() {
    echo ""
    info "Generating package files for all platforms..."
    echo ""

    local failed=0

    generate_homebrew || failed=$((failed + 1))
    generate_scoop || failed=$((failed + 1))
    generate_aur || failed=$((failed + 1))
    generate_binstall

    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    if [ $failed -eq 0 ]; then
        echo -e "${GREEN}✓ All package files generated in: $DIST_DIR/${NC}"
    else
        echo -e "${YELLOW}⚠ Generated with $failed error(s): $DIST_DIR/${NC}"
    fi
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    if [ -d "$DIST_DIR" ]; then
        echo -e "${GRAY}Generated files:${NC}"
        ls -la "$DIST_DIR/" 2>/dev/null || echo "  (directory empty)"
        if [ -d "$DIST_DIR/aur" ]; then
            echo ""
            echo -e "${GRAY}AUR files:${NC}"
            ls -la "$DIST_DIR/aur/" 2>/dev/null || echo "  (directory empty)"
        fi
    fi
    echo ""
}

# ============================================================================
# MAIN
# ============================================================================

# Parse arguments
PLATFORM=""
CHECK_ONLY=false

for arg in "$@"; do
    case "$arg" in
        --force)
            FORCE=true
            ;;
        --check)
            CHECK_ONLY=true
            ;;
        --help|-h|help)
            PLATFORM="help"
            ;;
        -*)
            error "Unknown option: $arg"
            echo ""
            show_help
            exit 1
            ;;
        *)
            if [ -z "$PLATFORM" ]; then
                PLATFORM="$arg"
            fi
            ;;
    esac
done

# Default to help if no platform specified
PLATFORM=${PLATFORM:-help}

# Handle check-only mode
if [ "$CHECK_ONLY" = true ]; then
    validate_prerequisites
    validate_release_exists
    show_summary
    exit 0
fi

# Handle help first (before validation)
if [ "$PLATFORM" = "help" ] || [ "$PLATFORM" = "--help" ] || [ "$PLATFORM" = "-h" ]; then
    show_help
    exit 0
fi

# Validate prerequisites for all other commands
validate_prerequisites

case $PLATFORM in
    crates)
        publish_crates
        ;;
    homebrew)
        generate_homebrew
        ;;
    scoop)
        generate_scoop
        ;;
    aur)
        generate_aur
        ;;
    binstall)
        generate_binstall
        ;;
    all)
        generate_all
        ;;
    status)
        show_status
        ;;
    *)
        error "Unknown platform: $PLATFORM"
        echo ""
        echo -e "${YELLOW}Available platforms:${NC} crates, homebrew, scoop, aur, binstall, all"
        echo ""
        echo -e "${GRAY}Run './scripts/publish.sh help' for more information${NC}"
        exit 1
        ;;
esac

# Show summary at the end
show_summary

# Exit with error if there were any errors
if [ ${#ERRORS[@]} -gt 0 ]; then
    exit 1
fi

exit 0
