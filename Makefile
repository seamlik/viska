# LTO due to https://github.com/rust-lang/rust/issues/50007
CARGO_ANDROID_COMMAND = CARGO_PROFILE_RELEASE_LTO=true cross build --package viska_android --release --target

# `--target-dir` due to https://github.com/rust-lang/cargo/issues/9137
CARGO_INSTALL_GITHUB_ARGS = --debug --target-dir=target

PRETTIER_ARGS = --ignore-path .gitignore .

verify:
	cargo fmt -- --check
	prettier --check $(PRETTIER_ARGS)
	gradle spotlessCheck
	cargo test

.PHONY: android-native
android-native:
	$(CARGO_ANDROID_COMMAND) aarch64-linux-android
	$(CARGO_ANDROID_COMMAND) armv7-linux-androideabi
	$(CARGO_ANDROID_COMMAND) x86_64-linux-android

# This target must be run at least once before building the project
.PHONY: prepare-local
prepare-local:
	diesel database reset --database-url file:/tmp/viska-sample-$(shell date +%s).db
	cargo riko

# For GitHub Actions
.PHONY: prepare-github
prepare-github:
	diesel database setup --database-url file:viska-sample.db
	diesel print-schema --database-url file:viska-sample.db > core/src/database/schema.rs
	cargo riko

.PHONY: prettier
prettier:
	prettier --write $(PRETTIER_ARGS)

# For installing build environment on GitHub Actions
.PHONY: install-env-github
install-env-github:
	npm install --global prettier @prettier/plugin-xml
	cargo install --git https://github.com/seamlik/riko.git --bin cargo-riko $(CARGO_INSTALL_GITHUB_ARGS)
	cargo install diesel_cli --no-default-features --features sqlite $(CARGO_INSTALL_GITHUB_ARGS)