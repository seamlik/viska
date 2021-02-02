export DATABASE_URL = file:/tmp/viska-sample-$(shell date +%s).db

# LTO due to https://github.com/rust-lang/rust/issues/50007
CARGO_ANDROID_COMMAND = CARGO_PROFILE_RELEASE_LTO=true cross build --package viska_android --release --target

PRETTIER_ARGS = --ignore-path .gitignore .

verify:
	cargo test
	gradle check
	prettier --check $(PRETTIER_ARGS)

.PHONY: android-native
android-native:
	$(CARGO_ANDROID_COMMAND) aarch64-linux-android
	$(CARGO_ANDROID_COMMAND) armv7-linux-androideabi
	$(CARGO_ANDROID_COMMAND) x86_64-linux-android

# This target must be run at least once before building the project
.PHONY: prepare
prepare: diesel-schema
	cargo riko

.PHONY: prettier
prettier:
	prettier --write $(PRETTIER_ARGS)

.PHONY: install-prettier
install-prettier:
	npm install --global prettier @prettier/plugin-xml

.PHONY: diesel-schema
diesel-schema:
	diesel database reset