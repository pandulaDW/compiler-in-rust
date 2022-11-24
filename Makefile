build:
	cargo build --release
	sudo cp ./target/release/compiler /usr/local/bin
	sudo mv /usr/local/bin/compiler /usr/local/bin/monkey