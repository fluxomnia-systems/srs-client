# Releasing

The current release preparation is **0.4.0**. Its package version, README, and
changelog are updated together. Publishing happens separately after those changes
are reviewed and merged into `main`.

## Prerequisites

- Install `cargo-release` and `git-cliff` (`cargo install cargo-release git-cliff`).
- Use a clean `main` checkout containing the release preparation.
- Have GitHub push access and a crates.io publishing credential configured through
  `cargo login` or `CARGO_REGISTRY_TOKEN`.
- Confirm the merged commit's CI passes, including the pinned SRS matrix.

## Validate the package

```bash
cargo test --all-targets
make cargo.fmt check=yes
make lint
make doc
lat check
cargo publish --dry-run
```

The three live tests are ignored by the first command. CI runs them against SRS
6.0.191, 7.0.162, and 8.0.48; `make test-http-api` runs one selected image locally.
A publish dry run builds the packaged crate but does not upload it.

## Publish the prepared 0.4.0 version

Do not bump the version again after merging this preparation. Preview the current
version, then execute the same release only when ready to publish:

```bash
make release.dry version=release
make release.current
```

`release.current` runs `cargo release release --execute`. It publishes the current
manifest version, creates the `v0.4.0` tag, and pushes the release. Cargo-release
asks for confirmation. The changelog hook skips dry runs and preserves an already
prepared version section, including its breaking-change notes. Future releases
prepend their new section without rewriting historical notes.

A pushed tag runs CI; it does **not** create a GitHub Release. After publication,
create the GitHub Release for the existing `v0.4.0` tag using its changelog notes.
Verify the crates.io package and docs.rs build before updating consumers.

## Future version bumps

These commands bump, generate release metadata, publish, tag, and push. The
preview command defaults to a patch bump and never executes the release.

```bash
make release.dry version=minor
make release.minor
# Or: make release.patch / make release.major
```

The Makefile uses cargo-release's positional levels and explicit `--execute`.
The hook honors its [documented `DRY_RUN` flag](https://github.com/crate-ci/cargo-release/blob/main/docs/reference.md#pre-release-hook), since hooks also run during previews.
README replacements update only crate badge/tag references and the dependency
line. They must preserve the separately pinned SRS server versions.

## Recovering an interrupted release

Inspect crates.io, the remote tag, and local changes before retrying. Published
crate versions are immutable; do not delete a published release tag or attempt to
overwrite the version. Use cargo-release's individual steps (`publish`, `tag`, or
`push`) only for the operations that have not completed.
