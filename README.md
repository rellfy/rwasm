# rwasm
Example project for building WASM applications in Rust with no dependencies.

### Building

```
rustup target add wasm32-unknown-unknown
cargo build --release --target=wasm32-unknown-unknown
```

### Serving
You can run a simple static server with cargo:
```
cargo install basic-http-server
basic-http-server .
```