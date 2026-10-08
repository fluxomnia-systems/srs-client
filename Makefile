###############################
# Common defaults/definitions #
###############################

comma := ,

# Checks two given strings for equality.
eq = $(if $(or $(1),$(2)),$(and $(findstring $(1),$(2)),\
                                $(findstring $(2),$(1))),1)

OS_NAME := $(shell uname -s)




###########
# Aliases #
###########

check: fmt lint doc

fmt: cargo.fmt

lint: cargo.lint

doc: cargo.doc

test: cargo.test

# Run HTTP API tests with local SRS docker-compose
#
# Usage:
#	make test-http-api [CARGO_TEST_ARGS="..."]

test-http-api:
	./scripts/test-http-api.sh $(CARGO_TEST_ARGS)

#############
# Release #
#############

release: release.patch

release.patch:
	@echo "Creating patch release..."
	cargo release patch --execute

release.minor:
	@echo "Creating minor release..."
	cargo release minor --execute

release.major:
	@echo "Creating major release..."
	cargo release major --execute

release.dry:
	@echo "Dry run release (no changes will be made)..."
	cargo release $(or $(version),patch)

release.current:
	cargo release release --execute

release.manual: release.check-changes release.update-version release.update-changelog release.commit release.tag release.push

release.check-changes:
	@echo "Checking for uncommitted changes..."
	@test -z "$$(git status --porcelain)" || (echo "Error: Working directory is not clean. Commit or stash changes first." && exit 1)
	@echo "Working directory is clean."

release.update-version:
	@echo "Updating version..."
	@read -p "Enter new version (current: $$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'): " new_version; \
	cargo set-version $$new_version

release.update-changelog:
	@echo "Updating changelog..."
	sh scripts/prepare-changelog.sh $$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')

release.commit:
	@echo "Committing release changes..."
	git add -A
	git commit -m "chore: prepare for release"

release.tag:
	@echo "Creating git tag..."
	git tag v$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')

release.push:
	@echo "Pushing to GitHub..."
	git push origin main --tags
	@echo "Don't forget to run 'cargo publish' to publish to crates.io"

publish:
	@echo "Publishing to crates.io..."
	cargo publish


##################
# Cargo commands #
##################

# Format Rust sources with rustfmt.
#
# Usage:
#	make cargo.fmt [check=(no|yes)]

cargo.fmt:
	cargo +nightly fmt --all $(if $(call eq,$(check),yes),-- --check,)


# Lint Rust sources with Clippy.
#
# Usage:
#	make cargo.lint

cargo.lint:
	cargo clippy --all -- -D clippy::pedantic -D warnings



# Generate Rust docs.
#
# Usage:
#	make cargo.doc

cargo.doc:
	cargo doc --all-features

# Run tests
#
# Usage:
#	make cargo.test

cargo.test:
	cargo nextest run


##################
# .PHONY section #
##################

.PHONY: fmt lint test publish release\
        release.patch release.minor release.major release.current release.dry release.manual\
        release.check-changes release.update-version release.update-changelog\
        release.commit release.tag release.push\
        cargo.fmt cargo.lint cargo.test test-http-api