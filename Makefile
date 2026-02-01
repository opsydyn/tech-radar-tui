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

tag-release:
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make tag-release VERSION=vX.Y.Z"; \
		exit 1; \
	fi
	git tag -a $(VERSION) -m "Release $(VERSION)"
	git push origin $(VERSION)

