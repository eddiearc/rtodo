exec_file: build
	cp $(shell pwd)/target/release/rtodo ./
build:
	cargo build --release
