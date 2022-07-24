# WIP Rust game

This monorepo contains the client & server crates for a WIP rust game.

## IDE & Analyzer

- This project is using VSCode and the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer
) extension
  - Make sure you have installed `clippy` and `rustfmt`
    - `rustup component add clippy`
    - `rustup component add rustfmt`
- Targetting other platforms:
  - To target WASM add `"rust-analyzer.cargo.target": "wasm32-unknown-unknown"` to the workspace/user settings
  - To target your local machine set it to whatever you local machine is:
    - Check with: `rustc --print cfg`, it's usually "{target_arch}-{target-vendor}-{target_os}-{target_env}"
      - E.g. mac os: `x86_64-apple-darwin`
      - E.g. linux gnu: `x86_64-unknown-linux-gnu`
    - Or just comment out the "rust-analyzer.cargo.target" line, analyzer should default to your system.