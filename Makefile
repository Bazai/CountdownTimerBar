APP := CountdownTimerBar
BUNDLE := releases/$(APP).app

.PHONY: build dist open stop run fmt fmt-check lint test check bump release agents license

# The bundle lands in releases/ (ignored by git).
build:
	./packaging/build-app.sh

# releases/CountdownTimerBar-v<version>.zip from the built bundle, for the GitHub release.
dist: build
	./packaging/dist.sh

open:
	open ./$(BUNDLE)

stop:
	-pkill -x $(APP)

run: build stop
	open ./$(BUNDLE)

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

# Warnings from our code are errors; the `block` note comes from a dependency (README).
lint:
	cargo clippy --all-targets --all-features --locked -- -D warnings

test:
	cargo test --locked

check: fmt-check lint test

# Bumps and commits the version: the next patch by default, `make bump BUMP=minor`,
# `make bump BUMP=major` or `make bump BUMP=1.2.3`. `DRY_RUN=1` only prints the plan.
BUMP ?= patch
bump:
	./packaging/bump.sh $(BUMP)

# Checks that the Cargo.toml version is ready to release. Pushing the bump to main
# lets the Release workflow tag and publish it.
release:
	./packaging/release.sh

# Links .agents/skills into .claude/skills and creates CLAUDE.md if it is missing.
agents:
	./packaging/agents.sh

# Sets the LICENSE years to <first year>-<current year>; the About card shows the same text.
license:
	./packaging/license.sh
