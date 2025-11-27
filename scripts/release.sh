#!/bin/bash
#
# Release script for idgen
# =========================
#
# This script automates the release process for idgen by:
# 1. Validating the environment (branch, clean working directory)
# 2. Updating the version in Cargo.toml
# 3. Running all tests to ensure quality
# 4. Committing and pushing the version bump
# 5. Creating and pushing a git tag to trigger GitHub Actions
#
# The GitHub Actions workflow (.github/workflows/release.yml) will then:
# - Build binaries for Linux, macOS, and Windows
# - Create a GitHub Release with the binaries attached
#
# Usage:
#   ./scripts/release.sh [version]
#
# Examples:
#   ./scripts/release.sh 1.5.0    # Release version 1.5.0
#   ./scripts/release.sh          # Interactive mode (prompts for version)
#
# Prerequisites:
#   - Git installed and configured
#   - Rust/Cargo installed
#   - Push access to the repository
#   - Must be on the master branch
#   - Working directory must be clean (no uncommitted changes)
#
# Version Format:
#   Use semantic versioning (MAJOR.MINOR.PATCH)
#   - MAJOR: Breaking changes
#   - MINOR: New features (backwards compatible)
#   - PATCH: Bug fixes (backwards compatible)
#
# After running this script:
#   1. GitHub Actions will build binaries (~5-10 minutes)
#   2. A GitHub Release will be created automatically
#   3. Edit the release notes at: https://github.com/maniartech/idgen/releases
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get version from argument or prompt
VERSION=$1

if [ -z "$VERSION" ]; then
    # Get current version from Cargo.toml
    CURRENT_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
    echo -e "${YELLOW}Current version: $CURRENT_VERSION${NC}"
    read -p "Enter new version (without 'v' prefix): " VERSION
fi

if [ -z "$VERSION" ]; then
    echo -e "${RED}Error: Version is required${NC}"
    exit 1
fi

TAG="v$VERSION"

echo -e "${GREEN}🚀 Releasing idgen $TAG${NC}"
echo ""

# Step 1: Ensure we're on master branch
echo -e "${YELLOW}[1/7] Checking branch...${NC}"
BRANCH=$(git branch --show-current)
if [ "$BRANCH" != "master" ]; then
    echo -e "${RED}Error: Must be on master branch (currently on '$BRANCH')${NC}"
    echo -e "${YELLOW}Tip: Run 'git checkout master' first${NC}"
    exit 1
fi
echo -e "${GREEN}✓ On master branch${NC}"

# Step 2: Ensure working directory is clean
echo -e "${YELLOW}[2/7] Checking working directory...${NC}"
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${RED}Error: Working directory has uncommitted changes${NC}"
    echo ""
    git status --short
    echo ""
    echo -e "${YELLOW}Tip: Commit or stash your changes first:${NC}"
    echo "  git add . && git commit -m 'your message'"
    echo "  or"
    echo "  git stash"
    exit 1
fi
echo -e "${GREEN}✓ Working directory clean${NC}"

# Step 3: Pull latest changes
echo -e "${YELLOW}[3/7] Pulling latest changes from origin...${NC}"
git pull origin master --quiet
echo -e "${GREEN}✓ Up to date with origin/master${NC}"

# Step 4: Update version in Cargo.toml
echo -e "${YELLOW}[4/7] Updating version in Cargo.toml...${NC}"
OLD_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
echo -e "${GREEN}✓ Version updated: $OLD_VERSION → $VERSION${NC}"

# Step 5: Run tests
echo -e "${YELLOW}[5/7] Running tests (this may take a moment)...${NC}"
if ! cargo test --quiet 2>/dev/null; then
    echo -e "${RED}Error: Tests failed! Fix the issues before releasing.${NC}"
    # Revert Cargo.toml change
    git checkout Cargo.toml
    exit 1
fi
TEST_COUNT=$(cargo test 2>&1 | grep -E "^test result:" | tail -1 | grep -oE "[0-9]+ passed" | grep -oE "[0-9]+")
echo -e "${GREEN}✓ All $TEST_COUNT tests passed${NC}"

# Step 6: Commit version bump
echo -e "${YELLOW}[6/7] Committing and pushing version bump...${NC}"
cargo update --quiet 2>/dev/null || true  # Update Cargo.lock
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to $VERSION" --quiet
git push origin master --quiet
echo -e "${GREEN}✓ Version bump committed and pushed${NC}"

# Step 7: Create and push tag
echo -e "${YELLOW}[7/7] Creating and pushing tag $TAG...${NC}"
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo -e "${YELLOW}Tag $TAG already exists. Deleting...${NC}"
    git tag -d "$TAG"
    git push origin --delete "$TAG" 2>/dev/null || true
fi
git tag "$TAG"
git push origin "$TAG"
echo -e "${GREEN}✓ Tag $TAG created and pushed${NC}"

echo ""
echo -e "${GREEN}🎉 Release $TAG initiated!${NC}"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "GitHub Actions is now building release binaries for:"
echo "  📦 Linux (x86_64-unknown-linux-musl)"
echo "  📦 macOS (x86_64-apple-darwin)"
echo "  📦 Windows (x86_64-pc-windows-msvc)"
echo ""
echo "This typically takes 5-10 minutes."
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📋 Next Steps:"
echo "  1. Watch build progress:"
echo "     https://github.com/maniartech/idgen/actions"
echo ""
echo "  2. Once complete, edit release notes:"
echo "     https://github.com/maniartech/idgen/releases/tag/$TAG"
echo ""
echo "  3. Suggested release notes template:"
echo ""
echo "     ## What's New in $TAG"
echo "     "
echo "     ### Features"
echo "     - Feature 1"
echo "     - Feature 2"
echo "     "
echo "     ### Bug Fixes"
echo "     - Fix 1"
echo "     "
echo "     ### Downloads"
echo "     | Platform | Binary |"
echo "     |----------|--------|"
echo "     | Linux    | idgen-linux-amd64 |"
echo "     | macOS    | idgen-macos-amd64 |"
echo "     | Windows  | idgen-windows-amd64.exe |"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
