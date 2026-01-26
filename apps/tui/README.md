
# 🛰️ Tech Radar ADR Generator (TUI)

A keyboard-driven TUI (Terminal User Interface) for creating **Architectural Decision Records (ADRs)** and **Tech Radar Blips** with quadrant + ring metadata, visualised on an animated radar.

Built using [`ratatui`](https://github.com/ratatui-org/ratatui), [`crossterm`](https://github.com/crossterm-rs/crossterm), and [`color-eyre`](https://docs.rs/color-eyre/). Optimised for speed, legibility, and no mouse interaction.

---

## ✨ Features

- Create **ADRs** and **Blips** with structured frontmatter
- Assign **Quadrant** (Platform, Language, Tool, Technique)
- Assign **Ring** (Hold, Assess, Trial, Adopt)
- Live **radar visualisation** of item location
- Animated **radar sweep**
- Built-in **keyboard shortcut help**
- Simple, clean interface for focused use

---

## 🧭 Controls

| Key        | Action                                     |
|------------|--------------------------------------------|
| `a`        | Start new ADR                              |
| `b`        | Start new Blip                             |
| `Enter`    | Confirm input                              |
| `Esc`      | Cancel or go back                          |
| `F1`       | Toggle help screen                         |
| `n`        | Start new entry (after completion)         |
| `q`        | Quit                                       |

---

## 🎯 Quadrants

| Key | Quadrant    | Description                                   |
|-----|-------------|-----------------------------------------------|
| `1` | Platforms   | Infra, APIs, hosting, platforms               |
| `2` | Languages   | Languages and frameworks                      |
| `3` | Tools       | Dev/test/ops tools                            |
| `4` | Techniques  | Practices, methods, engineering approaches    |

## 🌀 Rings

| Key | Ring    | Description                                                  |
|-----|---------|--------------------------------------------------------------|
| `1` | Hold    | Moving away from; avoid                                     |
| `2` | Assess  | Worth exploring                                              |
| `3` | Trial   | Try in a project; start learning                             |
| `4` | Adopt   | Use broadly; production-ready                                |

---

## 📂 Output

Generates `mdx` files in `./adrs/` with this format:

```mdx
---
title: "Rust"
date: 2025-03-23
status: "accepted"
quadrant: "languages"
ring: "adopt"
---

# Rust

## Context

...

## Decision

...

## Consequences

...

## References

...
```

For Blips, the structure is slightly different and omits `status`.

---

## 🛠️ Run

```bash
cargo run --release
```

---

## 🚧 Roadmap

- ⏳ Persistence (load/update existing entries)
- 🔍 Fuzzy search for ADRs
- 📈 Export radar as image or HTML
- 🗃️ Grouping/tagging support
- 🧠 AI-assisted context generation (optional)

---

## 🧬 Philosophy

> *"A good decision, made quickly, is better than a perfect decision made too late."*
> This tool enforces structure while maintaining flow. Built to support **deliberate development** and reduce decision fatigue.

---

## 🖼️ Screenshot

_Coming soon._

---

## 📜 License

MIT or Apache 2.0 — your choice.

---

## 💡 Tip

Use it as a CLI-only way to maintain **Tech Radar hygiene** and **ADR discipline** with visual feedback baked in.


examples that make sense for this:

---

**📛 badges worth adding**

| Badge | Why it matters |
|-------|----------------|
| ![Rust](https://img.shields.io/badge/Rust-🦀%20Rust-orange) | flex that it's built in Rust |
| ![License](https://img.shields.io/github/license/your-org/your-repo) | shows license at a glance |
| ![Crates.io](https://img.shields.io/crates/v/your-crate) | if you publish it on crates.io |
| ![Build](https://img.shields.io/github/actions/workflow/status/your-org/your-repo/ci.yml?branch=main) | shows build status |
| ![Lines of Code](https://tokei.rs/b1/github/your-org/your-repo?category=code) | total LoC (Tokei) |
| ![Last Commit](https://img.shields.io/github/last-commit/your-org/your-repo) | freshness indicator |
| ![Stars](https://img.shields.io/github/stars/your-org/your-repo?style=social) | social proof |
| ![Downloads](https://img.shields.io/crates/d/your-crate) | popularity on crates.io |
| ![Terminal UI](https://img.shields.io/badge/UI-TUI-blue) | optional, just to signal "Terminal UI" |
