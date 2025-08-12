# Dependencies and Features (MCP Server)

Add to `golem-cli/Cargo.toml`:

- Feature flag
```toml
[features]
mcp-serve = ["dep:rust-mcp-sdk", "dep:hyper"]
```

- Optional dependencies
```toml
[dependencies]
rust-mcp-sdk = { version = "0.5", default-features = false, features = ["server", "macros"], optional = true }
hyper = { version = "1", features = ["server"], optional = true }
```

Build locally without MCP (default features). Enable server for testing:
```bash
cargo run -p golem-cli --features mcp-serve -- --serve --serve-port 1232
```

Reference: `https://github.com/rust-mcp-stack/rust-mcp-sdk` 