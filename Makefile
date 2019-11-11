build:
	$(MAKE) -C frontend
	$(MAKE) -C backend
	cp backend/target/release/backend maps
