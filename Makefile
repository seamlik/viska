PRETTIER_ARGS = --ignore-path .gitignore .

verify: riko
	cargo test --package viska
	gradle check
	prettier --check $(PRETTIER_ARGS)

.PHONY: android-native
android-native:
	cross build --package viska_android --target aarch64-linux-android
	cross build --package viska_android --target aarch64-linux-android --release
	cross build --package viska_android --target armv7-linux-androideabi
	cross build --package viska_android --target armv7-linux-androideabi --release
	cross build --package viska_android --target x86_64-linux-android
	cross build --package viska_android --target x86_64-linux-android --release

# This target must be run at least once before building the project
.PHONY: riko
riko:
	cargo riko

.PHONY: prettier
prettier:
	prettier --write $(PRETTIER_ARGS)

.PHONY: install-prettier
install-prettier:
	npm install --global prettier @prettier/plugin-xml