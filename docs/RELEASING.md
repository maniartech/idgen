# Releasing idgen

The runbook for cutting a release and publishing it.

## TL;DR

```bash
./scripts/release.sh 1.5.0 --dry-run   # see what would happen
./scripts/release.sh 1.5.0             # tag it; GitHub Actions builds binaries
# wait ~5-10 min for the build to go green
./scripts/publish.sh crates            # publish to crates.io
./scripts/publish.sh all               # regenerate package manifests
```

Check where things stand at any point:

```bash
./scripts/publish.sh status
```

## Doing a release, start to finish

The full walkthrough. Replace `X.Y.Z` with the version you are releasing.

Only one step in here cannot be undone — step 6. Everything before it is
recoverable, and every step is safe to re-run.

### 1. Get master pushed and clean

```bash
cd /path/to/idgen
git status                 # must be clean
git push origin master     # release.sh will not run against an unpushed master
```

`release.sh` refuses to run with uncommitted changes or a master that has
diverged from origin. Both checks exist so the tag you push corresponds to
something other people can actually fetch.

### 2. Dry run

```bash
./scripts/release.sh X.Y.Z --dry-run
```

Changes nothing — no edits, no commits, no tags, no pushes. It prints each step
prefixed with `would run:` so you can see the plan.

If the version in `Cargo.toml` is already `X.Y.Z` (say you bumped it by hand in
an earlier PR), step 4 reports *"Already at X.Y.Z — nothing to change"*. That is
expected, not a problem.

### 3. Release

```bash
./scripts/release.sh X.Y.Z
```

Runs the tests with `--locked`, sets and commits the version if needed, pushes
master, then creates and pushes the `vX.Y.Z` tag. Pushing the tag is what
triggers the build.

If it fails partway — a test breaks, the network drops — fix the cause and run
the exact same command again. It picks up from wherever it got to.

### 4. Wait for the build

```bash
gh run watch                      # or: <repo>/actions
```

Takes roughly 5-10 minutes and produces four binaries: Linux, Windows, macOS
Intel, and macOS Apple Silicon.

**Do not skip ahead to step 7.** The manifests download these assets to compute
their SHA256 hashes, so they have to exist first.

If a build fails here, you have a pushed tag with an incomplete release. Fix the
cause and cut a *new* patch version — do not move the tag (see
[Tag conflicts](#tag-conflicts-are-not-resolved-for-you)).

### 5. Release notes

The workflow creates the GitHub Release automatically. Attach notes to it:

```bash
gh release edit vX.Y.Z --notes-file notes.md
```

Or edit in the browser at `<repo>/releases/tag/vX.Y.Z`. See
[Version numbers](#version-numbers) for what has to be called out — anything
that changes generated output belongs here, prominently.

### 6. Publish to crates.io

```bash
./scripts/publish.sh crates
```

Checks the registry, validates with `cargo publish --dry-run --locked`, then
**prompts for confirmation**. Nothing is uploaded until you answer `y`.

**This is the point of no return.** A version on crates.io can be yanked but
never replaced or re-uploaded. Everything up to here can be redone; this cannot.

Needs `cargo login` once, with a token from <https://crates.io/me> carrying the
`publish-new` and `publish-update` scopes.

### 7. Regenerate the package manifests

```bash
./scripts/publish.sh all
```

Writes the Homebrew formula, Scoop manifest, AUR PKGBUILDs, and binstall
metadata into `dist/packages/` (gitignored). Nothing is published — copy them
into your tap, bucket, or AUR checkout yourself.

If you see `PLACEHOLDER_SHA256` in the output, the assets were not downloadable
yet. Wait and re-run; the hashes fill in. Never hand-edit a placeholder.

### 8. Confirm

```bash
./scripts/publish.sh status
```

Expect: published on crates.io, tagged in git, assets available.

## Everything is idempotent

Both scripts are safe to re-run. If one fails halfway — a dropped connection, a
failed test, a Ctrl-C — fix the cause and run the same command again. Nothing
duplicates and nothing needs unwinding.

This is not achieved with a local ledger of "what I did last time". Each step
asks the thing that actually knows:

| Question | Source of truth |
|---|---|
| Is this version already tagged? | `git ls-remote origin` |
| Is master already pushed? | git |
| Is this version on crates.io? | the crates.io API |
| Which assets does a release build? | `.github/workflows/release.yml` |
| What is the crate/binary/version called? | `Cargo.toml` |

A previous version of `publish.sh` tracked state in `scripts/.publish-history`,
a gitignored, machine-local file. It was wrong in both directions: publish from
a second machine and it claimed nothing had ever shipped; delete it and the
script tried to re-publish a live version. The file is still written as a
convenience log, but nothing branches on it.

## Nothing is hardcoded

The crate is `idgen-cli`. The binary is `idgen`. Those are *different*, and
`idgen` is an unrelated crate on crates.io owned by someone else — so a script
that hardcodes the wrong one sends users to the wrong package. That is exactly
what the old `publish.sh` did: it printed `cargo install idgen`.

So `scripts/_common.sh` derives every name from the file that owns it, and the
scripts read from there. Consequences worth knowing:

- **Add a build target** to `.github/workflows/release.yml` and `publish.sh`
  picks it up automatically — the Homebrew formula, Scoop manifest, and binstall
  metadata all follow. No edit to the scripts.
- **Rename the crate or binary** in `Cargo.toml` and every message, formula, and
  URL follows.
- **A target the workflow does not build never appears in a manifest.** The old
  formula advertised an `idgen-macos-arm64` asset that was never built, so every
  Apple Silicon install failed against a 404.

## What release.sh does internally

Reference for the seven steps inside the script — distinct from the walkthrough
above, which is what *you* run. Each is re-runnable:

1. **Branch** — must be on `master`.
2. **Clean tree** — refuses to release uncommitted work.
3. **Sync** — fast-forward from `origin/master`. Refuses to merge if the two
   have diverged; cutting a release from a surprise merge is not automatic.
4. **Version** — sets it in `Cargo.toml`, or skips if it is already correct.
5. **Tests** — `cargo test --locked`, and refuses to continue if zero tests
   report as passed.
6. **Commit + push** — skips the commit if there is nothing staged.
7. **Tag + push** — triggers the release workflow.

Then GitHub Actions builds Linux, macOS (Intel and Apple Silicon), and Windows
binaries and attaches them to a GitHub Release.

### Tag conflicts are not resolved for you

If `vX.Y.Z` already exists and points at a *different* commit, the script stops.
It will not move the tag.

That is deliberate. A remote tag may already have a published Release with
binaries people have downloaded; moving it silently changes what that version
means for anyone who already installed it. The right answer is almost always to
release a new version. `--force` exists, and tells you exactly what it will
destroy before it does.

If the tag already points at `HEAD`, the script reports the release is already
done and exits successfully.

### It does not run `cargo update`

The old script ran `cargo update` at step 6 — *after* the tests at step 5. The
lockfile you tagged and shipped had therefore never been tested. When this was
found, that `cargo update` would have pulled **85 packages**, including a
major-version jump of `anstream`.

Dependency updates get their own commit and their own test run:

```bash
cargo update
cargo test
git commit -am "chore: update dependencies"
```

Tests run with `--locked`, so what gets tested is exactly what gets shipped.

## Publishing

Run after the release build is green — the manifests need the assets to exist so
their SHA256 hashes can be computed.

```bash
./scripts/publish.sh crates      # the only irreversible step
./scripts/publish.sh homebrew    # -> dist/packages/idgen.rb
./scripts/publish.sh scoop       # -> dist/packages/idgen.json
./scripts/publish.sh aur         # -> dist/packages/aur/PKGBUILD-{bin,src}
./scripts/publish.sh all         # all manifests
./scripts/publish.sh binstall    # prints Cargo.toml metadata
```

Only `crates` publishes anything. The rest write files under `dist/packages/`
(gitignored) for you to copy into a tap, bucket, or AUR checkout.

**crates.io is permanent.** A version can be yanked but never replaced or
re-uploaded. So `publish.sh crates`:

- checks the registry first and exits cleanly if the version is already live;
- **refuses to proceed if it cannot reach the registry**, rather than guessing;
- runs `cargo publish --dry-run --locked` before asking;
- prompts for confirmation.

There is no `--force` for crates. If a version is live, the only way forward is
a new version.

If a manifest shows `PLACEHOLDER_SHA256`, the assets were not downloadable yet.
Wait for the build and re-run — it is idempotent, and the hashes fill in. Never
hand-edit a placeholder to ship it.

## Version numbers

Semver, validated by the script — `1.5`, `v1.5.0`, and `1.5.0.0` are rejected.

- **MAJOR** — breaking changes
- **MINOR** — new features, or a change to generated output
- **PATCH** — bug fixes

The output-change case is easy to miss. v1.5.0 was a *minor*, not a patch,
because the v1 node-ID fix changed the bytes the tool emits: v1 UUIDs now carry
a random per-process node instead of a hardcoded `01:02:03:04:05:06`. No CLI
surface broke, but anyone storing v1s saw a real difference. Call that out in
the release notes.

## When something goes wrong

**"Tests failed"** — `Cargo.toml` is reverted automatically; nothing was
committed or tagged. Fix and re-run.

**"Cannot fast-forward from origin/master"** — local and remote diverged.
Reconcile by hand, then re-run.

**"No tests reported as passed"** — the suite ran but nothing passed, or the
counting broke. Run `cargo test` and look. Do not bypass this.

**"Could not reach crates.io"** — check connectivity and retry. The script will
not guess at publish state.

**Tag pushed but the build failed** — fix, then release a *new* patch version.
Don't move the tag.

**Published a broken version to crates.io** — it cannot be replaced. Yank it and
release a new one:

```bash
cargo yank --version 1.5.0 idgen-cli
```

Yanking stops new dependents from selecting it; it does not delete it, and
existing lockfiles keep working.
