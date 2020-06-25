PRETTIER_ARGS = --ignore-path .gitignore --plugin=@prettier/plugin-xml .

verify: riko
	cargo test --package viska
	gradle check
	prettier --check $(PRETTIER_ARGS)

.PHONY: android
android: $(DEMO_DATABASE_LOCATION) riko
	cross build --package viska_android --target aarch64-linux-android
	cross build --package viska_android --target x86_64-linux-android
	cross build --package viska_android --release --target aarch64-linux-android
	cross build --package viska_android --release --target x86_64-linux-android
	gradle assemble

.PHONY: riko
riko:
	cargo riko

.PHONY: prettier
prettier:
	prettier --write $(PRETTIER_ARGS)