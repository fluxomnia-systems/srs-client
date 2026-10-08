#!/bin/sh
# @lat: [[releases#Releases#Version preparation]]
set -eu

version=${1:?release version is required}
# cargo-release invokes hooks even when the release is only a preview.
if [ "${DRY_RUN:-false}" = "true" ]; then
    exit 0
fi
# Keep reviewed release notes, including breaking-change migration details.
if grep -Fq "## [$version]" CHANGELOG.md; then
    exit 0
fi
git-cliff --unreleased --tag "v$version" --prepend CHANGELOG.md
