# Releasing

The current release preparation is **0.4.0**. Its package version, README, and
changelog are updated together. Publishing happens separately after those changes
are reviewed and merged into `main`.

## One-time crates.io setup

Configure a [Trusted Publisher](https://crates.io/docs/trusted-publishing) in the
`srs-client` crate settings on crates.io with these exact values:

- GitHub owner: `fluxomnia-systems`
- Repository: `srs-client`
- Workflow filename: `publish.yml`
- Environment: `crates-io`

The workflow uses GitHub OIDC and the official crates.io authentication action;
no `CARGO_REGISTRY_TOKEN` repository secret is required. Only the final publish
job can request an OIDC token, after package validation and all live tests pass.
The action revokes its temporary token when the job ends.

## Prerequisites

- Install `cargo-release` and `git-cliff` (`cargo install cargo-release git-cliff`).
- Use a clean `main` checkout containing the release preparation.
- Have GitHub push access and the Trusted Publisher configuration above.

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

## Publish a release tag

Pushing a stable `vMAJOR.MINOR.PATCH` tag starts `.github/workflows/publish.yml`.
It requires the tag to match Cargo.toml and point to a commit on `main`. Every job
checks out the same resolved commit, even if a tag moves while validation runs.
Package checks and live tests against SRS 6.0.191, 7.0.162, and 8.0.48 must pass
before publishing. Publication runs are serialized per tag and never cancel an
in-progress publication.

For a prepared version that has not been tagged yet:

```bash
make release.dry version=release
make release.current
```

Cargo-release prepares and pushes the tag; `publish = false` in its configuration
leaves registry publication to the workflow. The changelog hook skips dry runs
and preserves reviewed notes. A GitHub Release remains a separate action.

## Publish the existing v0.4.0 tag

The tag predates the publish workflow. After the workflow is on `main` and the
Trusted Publisher is configured, use **Actions → Publish to crates.io → Run
workflow**, select `main`, and enter `v0.4.0`. The CLI equivalent is:

```bash
gh workflow run publish.yml --ref main -f tag=v0.4.0
```

Manual dispatch validates and publishes the existing tagged source, without
moving or recreating the tag. Verify the workflow succeeds and the version appears
on crates.io before updating consumers. Already published versions are immutable;
a second publish attempt fails rather than replacing the package.

## Future version bumps

These commands bump, generate release metadata, tag, and push. The pushed tag
triggers the publishing workflow. The preview command defaults to a patch bump and never executes the release.

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
overwrite the version. Retry the failed GitHub Actions run when publication has
not completed. If only
tagging or pushing failed locally, use cargo-release's corresponding individual
step. Do not rerun a version bump to retry the same release.
