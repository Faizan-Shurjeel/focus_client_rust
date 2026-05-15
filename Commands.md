### Correct command examples

Use these:

- `cargo run -- h`
- `cargo run -- analytics`
- `cargo run -- report`
- `cargo run --release -- h`
- `cargo run --release -- analytics`
- `cargo run --release -- report`

For dash-prefixed aliases, Cargo needs the separator:

- `cargo run --release -- --a`
- `cargo run --release -- --help`
- `cargo run --release -- --report`

Without the separator, Cargo eats the flag before your app sees it — that’s why `cargo run --release --a` showed Cargo’s error.

### Verified

I ran:

- `cargo run -- h` — now prints app help
- `cargo run --release -- --a` — runs analytics
- `cargo check`
- `cargo clippy --all-targets --all-features`
- `cargo test`

All clean.
