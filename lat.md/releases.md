# Releases

SRS client 0.4.0 is the breaking API release for the merged management, signaling, and callback upgrade. Release preparation and registry publication are separate steps.

## Version preparation

Cargo.toml, the README dependency and badge, and CHANGELOG.md carry the same crate version. SRS server versions stay pinned independently. The 0.4 migration guide documents changed models, errors, and deprecated helpers.

The hook in `scripts/prepare-changelog.sh` skips dry runs, preserves an already prepared release section, and prepends new sections without rewriting reviewed historical notes.

## Publication

After release preparation merges into a clean main checkout, `make release.dry version=release` previews the existing version and `make release.current` publishes it without another bump. RELEASE.md documents prerequisites and recovery.

Cargo-release uses explicit execution and positional bump levels. Changelog tags use the same `v` prefix as Git tags. README replacements target crate-version locations only. Tag CI validates the release; a GitHub Release is created separately.

## Verification

Package verification builds the archive with `cargo publish --dry-run`; contract tests, strict Clippy, nightly formatting, rustdoc, and lat validation check the prepared sources. Live API tests run in the existing pinned SRS CI matrix.
