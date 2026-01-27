# Tech Radar TUI

![Tech Radar mascot](tech_rdar_rat.webp)

Keyboard-driven terminal UI for creating Architectural Decision Records (ADRs) and Tech Radar blips with quadrant/ring metadata, radar visualization, and quick navigation.

## Features

- Create ADRs and blips with structured front matter
- Quadrant + ring selection with live radar placement
- Charts and radar visualizations
- Headless stats mode for CI or scripts

## Quick start

```bash
moon run tui:dev
```

Or run via Cargo:

```bash
cargo run --manifest-path apps/tui/Cargo.toml --release
```

## Headless stats

```bash
moon run tui:headless
```

## Install

Download the latest release binary for your platform from the GitHub Releases page and place it on your PATH:

<https://github.com/opsydyn/tech-radar-tui/releases>

- Linux: `tech-radar-tui-linux`
- macOS: `tech-radar-tui-macos`
- Windows: `tech-radar-tui-windows.exe`

Example (Linux/macOS):

```bash
curl -L -o tech-radar-tui https://github.com/opsydyn/tech-radar-tui/releases/download/v0.2.0/tech-radar-tui-linux
chmod +x tech-radar-tui
./tech-radar-tui
```

## Release checks and tags

```bash
make release-check
make tag-release VERSION=v0.2.0
```

## Project layout

- App: `apps/tui`
- DB + migrations: `apps/tui/src/db`
- UI: `apps/tui/src/ui`

For detailed controls and usage, see `apps/tui/README.md`.
