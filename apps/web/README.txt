Tech Radar Web (Ratzilla)

Local dev:
- cargo install --locked trunk
- rustup target add wasm32-unknown-unknown
- cd apps/web
- trunk serve

Data export:
- cargo run --release --manifest-path apps/tui/Cargo.toml --bin ratatui_adr-gen -- --export > apps/web/radar.json
