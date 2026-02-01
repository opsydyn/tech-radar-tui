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
	@echo "release-plz handles tags. Merge the Release PR instead."
