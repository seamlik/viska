DEMO_DATABASE_LOCATION = target/demo.realm

verify:
	cargo test --package viska
	gradle check
	eslint election

.PHONY: android
android: $(DEMO_DATABASE_LOCATION)
	cross build --package viska_android --target aarch64-linux-android
	cross build --package viska_android --target x86_64-linux-android
	cross build --package viska_android --release --target aarch64-linux-android
	cross build --package viska_android --release --target x86_64-linux-android
	gradle assemble

#
# End of public tasks
#

$(DEMO_DATABASE_LOCATION):
	node --experimental-modules electron/bin/demo-db.mjs $(DEMO_DATABASE_LOCATION)