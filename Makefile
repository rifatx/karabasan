.PHONY: build web serve test check clean
build:
	cargo build --release
web:
	cargo build --release --lib --target wasm32-unknown-unknown
	cp target/wasm32-unknown-unknown/release/karabasan.wasm web/karabasan.wasm
serve: web
	python3 -m http.server 8080 --bind 127.0.0.1 --directory web
test: build web
	cargo test
	node tools/parity.mjs
check:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	cargo clippy --lib --target wasm32-unknown-unknown -- -D warnings
clean:
	cargo clean
	rm -f web/karabasan.wasm
