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

tag-release:
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make tag-release VERSION=vX.Y.Z"; \
		exit 1; \
	fi
	git tag -a $(VERSION) -m "Release $(VERSION)"
	git push origin $(VERSION)
