# Client

This crate holds the client code for a WIP rust game. Everything is very experimental now so lots of things are pretty hacky.

## Requirements

1. Rust is required to run and build this project, install it [here](https://www.rust-lang.org/tools/install)
2. You need to make sure you have added the wasm32 target to rust: `rustup target add wasm32-unknown-unknown`
3. Make sure you install trunk to serve the wasm code: `cargo install --locked trunk`

## Running

Quick: Cmd + Shift + B shows overview of tasks

### WASM

**Development mode:** Builds the project and opens it in a new browser tab. Auto-reloads when the project changes.
```sh
trunk serve
```

The game can then be accessed at: http://localhost:8080

**Release mode:** Builds the project and places it into the `dist` folder. Serving this folder by deploying it to the web or by running a simple http server inside of it will then make the game accessible.
```sh
trunk build
```

### Desktop

Root of the non-wasm project is the `main.rs` file.

**Development mode:**

Running:
```sh
cargo run
```

**Release mode:**
Creates unoptimized (with debug info & all features) binary into `target/debug`:
```sh
cargo build
```

Creates optimized (with logging features disables) binary into `target/release`:
```sh
cargo build --release --no-default-features
```
