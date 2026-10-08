# Releases

SRS client 0.4.0 is the breaking API release for the merged management, signaling, and callback upgrade. Release preparation and registry publication are separate steps.

## Version preparation

Cargo.toml, the README dependency and badge, and CHANGELOG.md carry the same crate version. SRS server versions stay pinned independently. The 0.4 migration guide documents changed models, errors, and deprecated helpers.

The hook in `scripts/prepare-changelog.sh` skips dry runs, preserves an already prepared release section, and prepends new sections without rewriting reviewed historical notes.

## Publication

Stable version tags trigger `.github/workflows/publish.yml`. Manual dispatch from main handles existing tags such as v0.4.0. Both paths require matching package metadata and a tag commit contained in main.

The resolver emits one immutable commit SHA for all jobs. Package checks and the three-version SRS live matrix gate publication. Only the final `crates-io` environment job receives OIDC permission and exchanges it for a temporary registry token.

Crates.io must trust owner `fluxomnia-systems`, repository `srs-client`, workflow `publish.yml`, and environment `crates-io`. No long-lived registry secret is stored in GitHub. Actions are pinned to verified revisions.

Cargo-release has `publish = false`; its commands prepare and push tags while GitHub Actions owns registry publication. This avoids a local publish racing the tag workflow. Runs serialize per tag; a GitHub Release remains separate.

## Verification

Package verification builds the archive with `cargo publish --dry-run`; contract tests, strict Clippy, nightly formatting, rustdoc, and lat validation check the prepared sources.

Live API tests run in both the existing CI matrix and the publish workflow. Actionlint validates workflow syntax; local resolver checks cover missing tags, mismatched versions, and tags outside main.
