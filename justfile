# dexpr build recipes

# Run all tests
test:
    cargo test

# Run benchmarks
bench:
    cargo bench --bench my_benchmark

# Run the main program
run:
    cargo run --release

# --- Publish ---

# Publish dexpr to Gitea cargo registry
publish:
    cargo publish --registry gitea --allow-dirty

# --- WASM ---

# Build wasm package (web target)
wasm:
    cd wasm && wasm-pack build --target web --release --scope duhanbalci

# Build wasm package (bundler target, for npm)
wasm-bundler:
    cd wasm && wasm-pack build --target bundler --release --scope duhanbalci

# --- Editor ---

# Generate Lezer parser from grammar
editor-grammar:
    cd editor && npx @lezer/generator src/dexpr.grammar -o src/parser.js

# Build editor package (CodeMirror language support)
editor-build: editor-grammar
    cd editor && bun run build

# Build editor demo page
editor-demo: editor-build
    cd editor && bun run demo

# Publish editor package to Gitea npm registry
editor-publish: editor-build
    cd editor && npm publish

# Publish wasm package to Gitea npm registry
wasm-publish: wasm-bundler
    cd wasm/pkg && npm publish

# --- Combined ---

# Build everything (wasm + editor)
build-all: wasm editor-build

# Clean all build artifacts
clean:
    cargo clean
    rm -rf wasm/pkg wasm/target
    rm -rf editor/dist
