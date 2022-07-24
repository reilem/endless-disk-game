# Client

This crate holds the client code for a WIP rust game. Everything is very experimental now so lots of things are pretty hacky.

## Requirements

Rust is required to run and build this project, install it [here](https://www.rust-lang.org/tools/install).

## Setup

Run this to fetch all required npm development packages:
```sh
npm install
```

## Running

### WASM

Root of the WASM project is the `lib.rs` file.

**Development mode:** Builds the project and opens it in a new browser tab. Auto-reloads when the project changes.
```sh
npm start
```

The game can then be accessed at: http://localhost:3000

**Release mode:** Builds the project and places it into the `dist` folder. Serving this folder by deploying it to the web or by running a simple http server inside of it will then make the game accessible.
```sh
npm run build
```

### Desktop

Root of the non-wasm project is the `main.rs` file.

**Development mode:**

Running with debug logging:
```sh
npm run desktop
```

Running with only error logging:
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

## Testing

Run tests in a browser of your choice:

```sh
npm test -- --firefox
npm test -- --chrome
npm test -- --safari
```
