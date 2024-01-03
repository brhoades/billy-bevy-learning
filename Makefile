target/wasm32-unknown-unknown/release/%.wasm: $(wildcard ./bins/table-tennis/src/*.rs)
	cargo build --release --target wasm32-unknown-unknown

out/%.js: target/wasm32-unknown-unknown/release/%.wasm
	wasm-bindgen --out-dir ./out/ --target web $%

default: out/table-tennis.js
