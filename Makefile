DEMO_DATABASE_LOCATION = target/demo.realm

verify: eslint
	cargo test --package viska
	gradle check

.PHONY: android
android: $(DEMO_DATABASE_LOCATION)
	cross build --package viska_android --target aarch64-linux-android
	cross build --package viska_android --target x86_64-linux-android
	cross build --package viska_android --release --target aarch64-linux-android
	cross build --package viska_android --release --target x86_64-linux-android
	gradle assemble

# Download Node.js dependencies
.PHONY: node_modules
node_modules:
	npm i
	(cd electron; npm i)

eslint:
	eslint --ext .mjs electron/bin electron/lib

#
# End of public tasks
#

$(DEMO_DATABASE_LOCATION):
	node --experimental-modules electron/bin/demo-db.mjs $(DEMO_DATABASE_LOCATION)