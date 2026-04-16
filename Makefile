test:
	cargo test -- --nocapture

format-check:
	cargo fmt -- --check

format-fix:
	cargo fmt --all

build:
	cargo build

run:
	cargo build && cargo run
