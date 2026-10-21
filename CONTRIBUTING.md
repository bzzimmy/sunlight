Decrypted arm64 iOS IPAs come from [Roblox-DeployHistory-Updates/ios-runner](https://github.com/Roblox-DeployHistory-Updates/ios-runner/releases). Grab the build matching `crates/sunlight/src/offsets.rs`.

To convert the IPA, inject Sunlight, and launch Roblox on macOS:

```sh
cargo build
cargo run -p runtime -- path/to/Roblox.ipa target/debug/libsunlight.dylib
```

To run a script once in a game:

```sh
cargo run -p cli -- exec path/to/script.luau
```

Focus on fixing existing features rather than adding new ones. If there's a feature you want that isn't implemented yet, open an issue first so we can discuss the best way to integrate it.

Don't PR AI slop or untested code. Using AI is fine, just review what you send and keep it tight. Make sure `cargo fmt --all --check` and `cargo clippy --all-targets -- -D warnings` pass before opening a PR.
