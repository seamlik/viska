PRETTIER_ARGS = --ignore-path .gitignore --plugin=@prettier/plugin-xml .

verify:
	cargo test --package viska
	gradle check
	prettier --check $(PRETTIER_ARGS)

.PHONY: android
android:
	cross build --package viska_android --target aarch64-linux-android
	cross build --package viska_android --target x86_64-linux-android
	cross build --package viska_android --release --target aarch64-linux-android
	cross build --package viska_android --release --target x86_64-linux-android
	gradle assemble

# This target must be run at least once before building the project
.PHONY: riko
riko:
	cargo riko

.PHONY: prettier
prettier:
	prettier --write $(PRETTIER_ARGS)