#!/bin/bash
#
# Release script for idgen
# =========================
#
# Tags a release and lets GitHub Actions build the binaries.
#
#   ./scripts/release.sh 1.5.0        # release 1.5.0
#   ./scripts/release.sh              # prompt for the version
#   ./scripts/release.sh 1.5.0 --dry-run   # show what would happen, change nothing
#
# WHAT IT DOES
#   1. Validates the version, the branch, and the working tree
#   2. Syncs with origin/master
#   3. Sets the version in Cargo.toml (if it isn't already)
#   4. Runs the full test suite against the exact lockfile that will ship
#   5. Commits the version bump (only if there is something to commit)
#   6. Pushes master
#   7. Creates and pushes the tag, which triggers .github/workflows/release.yml
#
# The workflow then builds Linux, macOS (Intel + Apple Silicon), and Windows
# binaries and attaches them to a GitHub Release.
#
# IDEMPOTENCY
# -----------
# Every step is safe to re-run. Interrupt this script at any point — network
# drop, failed test, Ctrl-C — and just run it again with the same version. Each
# step checks the real state of the world before acting:
#
#   - Version already set in Cargo.toml?      -> skips the edit
#   - Nothing staged to commit?               -> skips the commit (not an error)
#   - Already pushed?                         -> push is a no-op
#   - Tag exists and points at this commit?   -> skips, reports already released
#   - Tag exists and points somewhere else?   -> STOPS. See below.
#
# A tag that already exists on a *different* commit is a genuine conflict, not
# something to paper over. The previous version of this script deleted the local
# and remote tag and recreated it. That is dangerous: the remote tag may already
# have a published GitHub Release and downloaded binaries attached to it, and
# moving it silently changes what a published version means for anyone who
# already installed it. This script refuses and makes you decide. `--force`
# still exists, but it tells you exactly what it is about to destroy first.
#
# WHAT IT DELIBERATELY DOES NOT DO
# --------------------------------
# It does not run `cargo update`. The old script did, at step 6 — *after* the
# tests at step 5. That meant the lockfile you tagged and shipped was never the
# lockfile you tested. At the time of writing, `cargo update` here would bump 85
# packages, including a major-version jump of anstream. Dependency updates are a
# deliberate act with their own commit and their own test run; they are not a
# side effect of cutting a release.
#
# Tests run with `--locked`, which fails if Cargo.lock is out of date rather
# than quietly rewriting it. What you test is exactly what you ship.
#
# PREREQUISITES
#   - On master, working tree clean, push access
#   - Cargo.lock committed and in sync with Cargo.toml
#
# AFTER RUNNING
#   1. Watch the build:   <repo>/actions   (~5-10 min)
#   2. Edit release notes: <repo>/releases/tag/vX.Y.Z
#   3. Publish to crates.io: ./scripts/publish.sh crates

set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_common.sh"

cd "$REPO_ROOT"

# ---------------------------------------------------------------------------
# Arguments
# ---------------------------------------------------------------------------
TARGET_VERSION=""
DRY_RUN=false
FORCE=false

for arg in "$@"; do
    case "$arg" in
        --dry-run) DRY_RUN=true ;;
        --force)   FORCE=true ;;
        --help|-h)
            sed -n '2,60p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        -*) die "Unknown option: $arg (try --help)" ;;
        *)  [ -z "$TARGET_VERSION" ] && TARGET_VERSION="$arg" ;;
    esac
done

if [ -z "$TARGET_VERSION" ]; then
    echo "${YELLOW}Current version: $VERSION${NC}"
    read -r -p "Enter new version (without 'v' prefix): " TARGET_VERSION
fi

[ -n "$TARGET_VERSION" ] || die "Version is required"

# Strip an accidental leading v so `release.sh v1.5.0` behaves.
TARGET_VERSION="${TARGET_VERSION#v}"

is_semver "$TARGET_VERSION" || die \
    "'$TARGET_VERSION' is not valid semver (expected MAJOR.MINOR.PATCH, e.g. 1.5.0)"

TAG="v$TARGET_VERSION"

$DRY_RUN && echo "${BLUE}${BOLD}DRY RUN${NC} ${BLUE}- nothing will be changed, committed, or pushed${NC}"
echo "${GREEN}${BOLD}Releasing $CRATE_NAME $TAG${NC}"
echo ""

# run <description> <command...> — honours --dry-run uniformly, so there is one
# place where "would do" and "does" diverge rather than a $DRY_RUN check
# scattered through every step.
run() {
    local desc=$1; shift
    if $DRY_RUN; then
        echo "${GRAY}   would run: $*${NC}"
    else
        "$@" || die "$desc failed: $*"
    fi
}

# ---------------------------------------------------------------------------
# [1/7] Branch
# ---------------------------------------------------------------------------
echo "${YELLOW}[1/7] Checking branch...${NC}"
BRANCH="$(git branch --show-current)"
if [ "$BRANCH" != "master" ]; then
    die "Must be on master (currently on '$BRANCH')
$(hint "git checkout master")"
fi
success "On master"

# ---------------------------------------------------------------------------
# [2/7] Clean tree
# ---------------------------------------------------------------------------
echo "${YELLOW}[2/7] Checking working directory...${NC}"
if [ -n "$(git status --porcelain)" ]; then
    echo ""
    git status --short
    echo ""
    die "Working directory has uncommitted changes. Commit or stash them first."
fi
success "Working directory clean"

# ---------------------------------------------------------------------------
# [3/7] Sync with origin
# ---------------------------------------------------------------------------
echo "${YELLOW}[3/7] Syncing with origin...${NC}"
run "git fetch" git fetch --quiet origin
if ! $DRY_RUN; then
    # --ff-only: if master and origin/master have diverged, stop. Cutting a
    # release from a surprise merge commit is not something to do silently.
    git pull --ff-only origin master --quiet \
        || die "Cannot fast-forward from origin/master — local and remote have diverged.
$(hint "Reconcile them yourself, then re-run. This script will not merge for you.")"
fi
success "Up to date with origin/master"

# ---------------------------------------------------------------------------
# [4/7] Version (idempotent)
# ---------------------------------------------------------------------------
echo "${YELLOW}[4/7] Setting version in Cargo.toml...${NC}"
if [ "$VERSION" = "$TARGET_VERSION" ]; then
    # Already correct — someone bumped it by hand, or a previous run of this
    # script got this far before failing. Either way there is nothing to do,
    # and that is a success, not an error.
    success "Already at $TARGET_VERSION — nothing to change"
else
    run "version bump" sed -i "s/^version = \".*\"/version = \"$TARGET_VERSION\"/" Cargo.toml
    if ! $DRY_RUN; then
        # Re-read rather than trust the sed. A silently-failed bump that still
        # tags would ship the wrong version number.
        actual="$(cargo_field '[package]' 'version')"
        [ "$actual" = "$TARGET_VERSION" ] \
            || die "Version bump did not apply (Cargo.toml still reads '$actual')"
        # Keep Cargo.lock's record of our own package in step with Cargo.toml.
        # Scoped to this package only — this is not a dependency update.
        cargo update --offline --quiet --package "$CRATE_NAME" 2>/dev/null || true
    fi
    success "Version: $VERSION -> $TARGET_VERSION"
fi

# ---------------------------------------------------------------------------
# [5/7] Tests
# ---------------------------------------------------------------------------
echo "${YELLOW}[5/7] Running tests (--locked, this may take a moment)...${NC}"
if $DRY_RUN; then
    echo "${GRAY}   would run: cargo test --locked${NC}"
else
    TEST_OUTPUT="$(cargo test --locked 2>&1)" || {
        echo "$TEST_OUTPUT" | tail -30
        # Leave no half-applied bump behind on failure.
        git checkout -- Cargo.toml Cargo.lock 2>/dev/null || true
        die "Tests failed. Fix them before releasing. (Cargo.toml reverted.)"
    }
    TEST_COUNT="$(echo "$TEST_OUTPUT" | count_passed_tests)"
    # A zero count means the counting broke or nothing ran. Either way this is
    # not a green light — the old script printed "All 0 tests passed" and
    # sailed on.
    [ "$TEST_COUNT" -gt 0 ] \
        || die "No tests reported as passed — refusing to release.
$(hint "Run 'cargo test' by hand and check the output.")"
    success "All $TEST_COUNT tests passed"
fi

# ---------------------------------------------------------------------------
# [6/7] Commit + push (idempotent)
# ---------------------------------------------------------------------------
echo "${YELLOW}[6/7] Committing and pushing...${NC}"
if ! $DRY_RUN; then
    git add Cargo.toml Cargo.lock
    if git diff --cached --quiet; then
        # Nothing staged: the version was already committed by a previous run
        # or by hand. `git commit` would exit 1 here and, under `set -e`, abort
        # a release that is in fact fine.
        success "Nothing to commit — version already committed"
    else
        git commit --quiet -m "chore: release $TAG"
        success "Committed version bump"
    fi
    # Idempotent by nature: a no-op when already up to date.
    git push --quiet origin master || die "Failed to push master"
    success "Pushed master"
else
    echo "${GRAY}   would commit (if needed) and push master${NC}"
fi

# ---------------------------------------------------------------------------
# [7/7] Tag (idempotent, and refuses to move an existing tag)
# ---------------------------------------------------------------------------
echo "${YELLOW}[7/7] Tagging $TAG...${NC}"
HEAD_SHA="$(git rev-parse HEAD)"
LOCAL_TAG_SHA="$(git rev-parse -q --verify "refs/tags/$TAG^{commit}" 2>/dev/null || true)"
REMOTE_TAG_SHA="$(git ls-remote --tags origin "refs/tags/$TAG^{}" 2>/dev/null | awk '{print $1}' | head -1)"
[ -n "$REMOTE_TAG_SHA" ] || REMOTE_TAG_SHA="$(git ls-remote --tags origin "refs/tags/$TAG" 2>/dev/null | awk '{print $1}' | head -1)"

tag_conflict() {
    local where=$1 sha=$2
    echo ""
    warn "$TAG already exists on $where and points at a different commit:"
    echo "     existing: $sha  $(git log -1 --format='%s' "$sha" 2>/dev/null || echo '(unknown commit)')"
    echo "     HEAD:     $HEAD_SHA  $(git log -1 --format='%s' HEAD)"
    echo ""
    if ! $FORCE; then
        die "Refusing to move an existing tag.
$(hint "A published release may already be attached to it; moving it changes what")
$(hint "$TAG means for anyone who already installed it.")
$(hint "Release a new version instead (preferred), or re-run with --force.")"
    fi
    warn "--force given: the existing $TAG will be DELETED and recreated."
    warn "Any GitHub Release attached to it may be invalidated."
}

if [ -n "$REMOTE_TAG_SHA" ] && [ "$REMOTE_TAG_SHA" != "$HEAD_SHA" ]; then
    tag_conflict "origin" "$REMOTE_TAG_SHA"
elif [ -n "$LOCAL_TAG_SHA" ] && [ "$LOCAL_TAG_SHA" != "$HEAD_SHA" ]; then
    tag_conflict "this machine" "$LOCAL_TAG_SHA"
fi

if [ "$REMOTE_TAG_SHA" = "$HEAD_SHA" ] && [ "$LOCAL_TAG_SHA" = "$HEAD_SHA" ]; then
    success "$TAG already points at HEAD — already released, nothing to do"
    ALREADY_TAGGED=true
else
    ALREADY_TAGGED=false
    if $DRY_RUN; then
        echo "${GRAY}   would create and push tag $TAG at $HEAD_SHA${NC}"
    else
        if $FORCE && [ -n "$LOCAL_TAG_SHA" ]; then
            git tag -d "$TAG" >/dev/null
            git push origin --delete "$TAG" 2>/dev/null || true
        fi
        # Annotated: carries an author and a date, which lightweight tags do not.
        git tag -a "$TAG" -m "Release $TAG" 2>/dev/null || true
        git push --quiet origin "$TAG" || die "Failed to push tag $TAG"
        success "Tag $TAG created and pushed"
    fi
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
if $DRY_RUN; then
    echo "${BLUE}${BOLD}Dry run complete — nothing was changed.${NC}"
    echo "${GRAY}Re-run without --dry-run to release.${NC}"
    exit 0
fi

if $ALREADY_TAGGED; then
    echo "${GREEN}${BOLD}$TAG was already released.${NC}"
else
    echo "${GREEN}${BOLD}Release $TAG initiated.${NC}"
fi
echo ""
echo "GitHub Actions is building:"
release_assets | sed 's/^/  - /'
echo ""
echo "Next:"
echo "  1. Watch the build:    $REPO_URL/actions"
echo "  2. Edit release notes: $REPO_URL/releases/tag/$TAG"
echo "  3. Publish to crates.io once the build is green:"
hint "./scripts/publish.sh crates"
echo ""
