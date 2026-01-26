.PHONY: moon-build moon-check moon-format moon-lint moon-test moon-dev moon-release dev release fmt headless

moon-build:
	moon run tui:build

moon-check:
	moon run tui:check

moon-format:
	moon run tui:format

moon-lint:
	moon run tui:lint

moon-test:
	moon run tui:test

moon-dev:
	moon run tui:dev

moon-release:
	moon run tui:release

dev:
	moon run tui:dev
release:
	moon run tui:release

fmt:
	cargo fmt --manifest-path apps/tui/Cargo.toml

headless:
	cargo run --manifest-path apps/tui/Cargo.toml --bin ratatui_adr-gen

