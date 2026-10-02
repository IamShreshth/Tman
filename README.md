# tman

> A context-aware interactive layer over your terminal.

[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/IamShreshth/Tman)
[![Shell](https://img.shields.io/badge/shell-zsh%20%7C%20bash-green.svg)](https://github.com/IamShreshth/Tman)
[![Dependencies](https://img.shields.io/badge/dependencies-zero_runtime-brightgreen.svg)](requirements.txt)

`tman` drops an interactive HUD directly over your active shell prompt with `Ctrl+Shift+T` (or `Ctrl+Space`), enabling directory navigation, workflow execution, history search, and bookmarks without context switching.

---

## Installation

### Single-Step Install

```bash
curl -fsSL https://raw.githubusercontent.com/IamShreshth/Tman/main/install.sh | bash
```

Reload your shell to activate:

```bash
source ~/.zshrc   # If using Zsh
# OR
source ~/.bashrc  # If using Bash
```

Launch by pressing `Ctrl+Shift+T` (or `Ctrl+Space`), or run `tman`.

---

## Requirements

`tman` is written in Rust and compiles to a standalone, zero-dependency native binary. No Python, Node.js, or external runtimes are required.

- **OS**: macOS (Apple Silicon & Intel) or Linux (x86_64, aarch64)
- **Shell**: Zsh (v5.0+) or Bash (v4.0+)
- **Build tool**: Rust 1.70+ and Cargo (only if building from source)

See [requirements.txt](requirements.txt) for technical specifications.

---

## Build from Source

```bash
git clone https://github.com/IamShreshth/Tman.git
cd Tman
./install.sh
```

Or install directly via Cargo:

```bash
cargo install --git https://github.com/IamShreshth/Tman.git --locked
eval "$(tman init zsh)"  # Add to ~/.zshrc
```

---

## Keybindings

| Key | Context | Action |
| :--- | :--- | :--- |
| `Ctrl+Shift+T` / `Ctrl+Space` | Shell | Toggle HUD overlay |
| `Up` / `Down` or `j` / `k` | Menus | Move selection |
| `Enter` | Any | Execute action / `cd` into directory |
| `Right` / `l` | File Browser | Open selected directory |
| `Left` / `h` / `Backspace` | Submenus | Go to parent directory / back |
| `b` | File Browser | Pin selected folder to bookmarks |
| `d` or `x` | Bookmarks | Delete highlighted bookmark |
| `Tab` | Browser / Runners | Open action menu / insert command into shell |
| `/` | Lists | Filter with fuzzy search |
| `.` | File Browser | Toggle hidden files |
| `m` | Any | Return to main menu |
| `Esc` | Any | Close modal / exit HUD |
| `Ctrl+C` / `Ctrl+D` | Any | Force quit |

---

## Uninstallation

```bash
rm -f ~/.local/bin/tman
rm -rf ~/.config/tman
```

Remove the `eval "$(tman init ...)"` line from your `~/.zshrc` or `~/.bashrc`.

## License

Licensed under the [MIT License](LICENSE).

