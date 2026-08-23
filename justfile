check:
    cargo c
    cargo c --target=aarch64-apple-ios
    cargo c --target=aarch64-apple-ios-sim
    cargo fmt -- --check
    cargo clippy
    cargo clippy --target=aarch64-apple-ios
    cargo clippy --target=aarch64-apple-ios-sim
    cargo test

publish:
    cargo publish
