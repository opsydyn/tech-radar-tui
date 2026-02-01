dev:
	moon run tui:dev

release:
	moon run tui:release

fmt:
	cargo fmt --all

headless:
	cargo run --manifest-path apps/tui/Cargo.toml --release -- --headless

release-check:
	moon run tui:check
	moon run tui:test
	moon run tui:build

web-export:
	cargo run --release --manifest-path apps/tui/Cargo.toml --bin ratatui_adr-gen -- --export > apps/web/radar.json

web-serve:
	cd apps/web && trunk serve

# Release-plz handles tags via its workflow.
tag-release:
	@echo "release-plz handles tags. Merge the Release PR instead."

