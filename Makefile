export DATABASE_URL = file:/tmp/viska-sample-$(shell date +%s).db

PRETTIER_ARGS = --ignore-path .gitignore .

verify:
	cargo test
	gradle check
	prettier --check $(PRETTIER_ARGS)

.PHONY: android-native
android-native:
	cross build --package viska_android --target aarch64-linux-android --release
	cross build --package viska_android --target armv7-linux-androideabi --release
	cross build --package viska_android --target x86_64-linux-android --release

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