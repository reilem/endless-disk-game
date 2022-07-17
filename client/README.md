# Client

This crate holds the client code for a WIP rust game. Everything is very experimental now so lots of things are pretty hacky.

## Setup

```sh
npm install
```

### IDE & Analyzer

- This project uses VSCode as IDE and [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer
) as linter / analyzer.
- This is a bit hacky, but to get the analyzer to work for different targets I currently just do:
  - To target WASM add `"rust-analyzer.cargo.target": "wasm32-unknown-unknown"` to your user settings
  - To target your local machine set it to whatever you local machine is:
    - Check with: `rustc --print cfg`, it's usually "{target_arch}-{target-vendor}-{target_os}-{target_env}"
      - E.g. mac os: `x86_64-apple-darwin`
      - E.g. linux gnu: `x86_64-unknown-linux-gnu`
    - Or just comment out the "rust-analyzer.cargo.target" line, analyzer should default to your system.

## Running

### WASM

Root of the WASM project is the `lib.rs` file.

Debug mode: Builds the project and opens it in a new browser tab. Auto-reloads when the project changes.
```sh
npm start
```

The game can then be accessed at: http://localhost:3000

Release mode: Builds the project and places it into the `dist` folder.
```sh
npm run build
```

### Non-WASM

Root of the non-wasm project is the `main.rs` file.

Running with debug logging:
```sh
npm run desktop
```

Running with only error logging:
```sh
cargo run
```

## Testing

Run tests in a browser of your choice:

```sh
npm test -- --firefox
npm test -- --chrome
npm test -- --safari
```
