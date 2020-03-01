DEMO_DATABASE_LOCATION = target/demo.realm

# Build Android
android: $(DEMO_DATABASE_LOCATION)
	android/build.sh
	gradle assemble

# Generate a database for demo
.PHONY: demo-db
demo-db:
	node --experimental-modules electron/bin/demo-db.mjs $(DEMO_DATABASE_LOCATION)

# Download Node.js dependencies
.PHONY: node_modules
node_modules:
	npm i
	(cd electron; npm i)

#
# End of public tasks
#