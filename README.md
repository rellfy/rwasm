# rwasm

[![Github actions](https://github.com/rellfy/rwasm/workflows/Build/badge.svg)](https://github.com/rellfy/rwasm/actions)
[![Crate version](https://img.shields.io/crates/v/rwasm.svg)](https://crates.io/crates/rwasm)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/rellfy/rwasm/blob/master/LICENSE)

A minimalistic WASM library for Rust.

## Core concepts

### Remote Procedure Call
All user-defined communication from Rust to Javascript happens through remote procedure calls.
The RPCs to be executed are given to the `rwasm` JS instance as regular functions expecting a single `Uint8Array` parameter.
The user may send the name of the function to be executed along with the data as bytes.
There are two types of RPCs that can be executed through `rwasm`, described below.

### "send" RPC
The `send` RPC type sends data to Javascript and does not expect to receive anything back.

### "request" RPC
The `request` RPC type sends data to Javascript and expects to receive back an `usize` value.
The `usize` value represents the length of the data, in bytes, written to a Buffer identified by an `u32` value, which must be given as a parameter in the `request` RPC.

#### Data buffers
The `rwasm` library defines a hashmap of type `<u32, [u8; BUFFER_SIZE]>` (where `BUFFER_SIZE` is a constant equals to 128k) and allows insertion of new values as well as retrieval of a pointer to a value identified by the `u32` key, which can be called from Javascript and is used to write data to the specific buffer as requested by a `request` RPC.

## Building

```
rustup target add wasm32-unknown-unknown
cargo build --release --target=wasm32-unknown-unknown
```

## Serving
You can run a simple static server with cargo:
```
cargo install basic-http-server
basic-http-server .
```
