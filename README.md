<div align="center">

# tman

**A context-aware interactive layer over your terminal.**

[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/IamShreshth/Tman)
[![Shell](https://img.shields.io/badge/shell-zsh%20%7C%20bash%20%7C%20fish-green.svg)](https://github.com/IamShreshth/Tman)
[![Dependencies](https://img.shields.io/badge/dependencies-zero_runtime-brightgreen.svg)](docs/requirements.txt)

<br/>

<img src="docs/preview.png" alt="tman interface preview" width="850">

</div>

`tman` drops an interactive HUD directly over your shell prompt with `Ctrl+Shift+T` (or `Ctrl+Space`), enabling directory navigation, project workflow execution, history search, and system telemetry without interrupting your active workflow.

---

## Quick Start

### 1. Install
```bash
curl -fsSL https://raw.githubusercontent.com/IamShreshth/Tman/main/files/install.sh | bash
```

### 2. Activate
```bash
source ~/.zshrc   # If using Zsh
# OR
source ~/.bashrc  # If using Bash
```

### 3. Launch
Press `Ctrl+Shift+T` (or `Ctrl+Space`) anywhere in your shell, or run `tman`.

---

## Linux Installation

### Universal One-Liner (All Distros)

Automatically detects your architecture (`x86_64` / `aarch64`), libc (`glibc` / `musl`), downloads the pre-built binary (or builds via Cargo if needed), and configures your shell:

```bash
curl -fsSL https://raw.githubusercontent.com/IamShreshth/Tman/main/files/install.sh | bash
```

---

### 1. Arch Linux (Manjaro, EndeavourOS, CachyOS)

```bash
# 1. Install build prerequisites & optional clipboard/desktop helpers
sudo pacman -S --needed --noconfirm base-devel git rust wl-clipboard xclip xdg-utils

# 2. Install tman (via installer script or Cargo)
curl -fsSL https://raw.githubusercontent.com/IamShreshth/Tman/main/files/install.sh | bash
# OR build directly via Cargo:
# cargo install --git https://github.com/IamShreshth/Tman.git --locked

# 3. Activate in your shell:
source ~/.bashrc                         # Bash
# source ~/.zshrc                        # Zsh
# tman init fish | source                # Fish (append to ~/.config/fish/config.fish)
```

---

### 2. Fedora (RHEL, CentOS Stream, Nobara)

```bash
# 1. Install Rust toolchain & optional clipboard/desktop helpers
sudo dnf install -y gcc git cargo rust wl-clipboard xclip xdg-utils

# 2. Install tman (via installer script or Cargo)
curl -fsSL https://raw.githubusercontent.com/IamShreshth/Tman/main/files/install.sh | bash
# OR build directly via Cargo:
# cargo install --git https://github.com/IamShreshth/Tman.git --locked

# 3. Activate in your shell:
source ~/.bashrc                         # Bash
# source ~/.zshrc                        # Zsh
# tman init fish | source                # Fish (append to ~/.config/fish/config.fish)
```

---

### 3. Ubuntu / Debian (Linux Mint, Pop!_OS, Kali)

```bash
# 1. Install build tools, Rust toolchain & optional clipboard/desktop helpers
sudo apt update && sudo apt install -y build-essential curl git cargo rustc wl-clipboard xclip xdg-utils

# 2. Install tman (via installer script or Cargo)
curl -fsSL https://raw.githubusercontent.com/IamShreshth/Tman/main/files/install.sh | bash
# OR build directly via Cargo:
# cargo install --git https://github.com/IamShreshth/Tman.git --locked

# 3. Activate in your shell:
source ~/.bashrc                         # Bash
# source ~/.zshrc                        # Zsh
# tman init fish | source                # Fish (append to ~/.config/fish/config.fish)
```

> **Tip for older Ubuntu/Debian releases:** If the `apt` Rust version is older than `1.70`, install the latest toolchain via `rustup`:
> ```bash
> curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
> source "$HOME/.cargo/env"
> ```

---

### Manual Shell Hook Setup (If Installing via Cargo)

If you installed via `cargo install` instead of `install.sh`, add the shell hook for your shell:

| Shell | Command |
| :--- | :--- |
| **Bash** | `echo 'eval "$(tman init bash)"' >> ~/.bashrc && source ~/.bashrc` |
| **Zsh** | `echo 'eval "$(tman init zsh)"' >> ~/.zshrc && source ~/.zshrc` |
| **Fish** | `mkdir -p ~/.config/fish && echo 'tman init fish \| source' >> ~/.config/fish/config.fish && source ~/.config/fish/config.fish` |

> **Note on Linux keybindings:** Most Linux terminal emulators intercept `Ctrl+Shift+T` to open a new tab. `tman` automatically binds **`Ctrl+Space`** alongside `Ctrl+Shift+T` so you can press `Ctrl+Space` out of the box on GNOME Terminal, Konsole, XFCE Terminal, Alacritty, Kitty, and WezTerm.

---

## Capabilities

- **Zero-Daemon HUD**: Ephemeral full-screen overlay that takes ownership of the terminal buffer and releases it cleanly on exit.
- **System Telemetry**: Real-time cross-platform inspection of battery status, CPU load averages, memory allocation, volume capacity, and active Git branch state (native sysfs & procfs on Linux, sysctl & pmset on macOS).
- **Workflow Discovery**: Automatically senses repository types (Rust, Python, Node, Go, Docker, Make, CMake, Just) and compiles actionable targets.
- **Navigation & Actions**: Fast directory traversal with persistent directory pinning, in-prompt `cd`, clipboard export (`wl-copy`, `xclip`, `pbcopy`), and desktop app opening (`xdg-open` on Linux, `open` on macOS).

---

## Keybindings

| Key | Context | Action |
| :--- | :--- | :--- |
| `Ctrl+Shift+T` / `Ctrl+Space` | Shell | Toggle HUD overlay (`Ctrl+Space` recommended on Linux) |
| `Up` / `Down` or `j` / `k` | Navigation | Move cursor selection |
| `Enter` | Any View | Execute selected action / `cd` into folder |
| `Right` / `l` | File Browser | Open highlighted directory |
| `Left` / `h` / `Backspace` | Submenus | Navigate to parent directory / previous view |
| `b` | Menu / Files | Bookmark current directory or highlighted folder |
| `d` or `x` | Bookmarks | Delete highlighted bookmark |
| `Space` | Git Changed Files | Stage / unstage highlighted file in-place |
| `Q` | File Browser | Quit HUD and `cd` shell into the browsed directory |
| `Tab` | Browser / Runners | Open action menu (`Open with app`, `Open file manager`, copy path) |
| `/` | Lists | Filter items with multi-word fuzzy search |
| `.` | File Browser | Toggle hidden dotfiles |
| `m` | Submenus | Return to main menu |
| `Esc` | Any View | Clear search / close modal / exit HUD |
| `Ctrl+C` / `Ctrl+D` | Any View | Terminate overlay |

---

## Requirements

`tman` is written in Rust and compiles to a standalone, zero-dependency native binary. No external runtimes (Python, Node.js) are needed.

| Component | Requirement |
| :--- | :--- |
| **Operating System** | macOS (Apple Silicon & Intel) or Linux (x86_64, aarch64, musl/glibc) |
| **Supported Shells** | Zsh (v5.0+), Bash (v4.0+), or Fish (v3.4+) |
| **Build Toolchain** | Rust 1.70+ and Cargo (only when building from source) |

See [linux.md](linux.md) for Linux-specific details and [docs/requirements.txt](docs/requirements.txt) for technical specifications.

---

## Alternative Installation

### Install via Cargo
```bash
cargo install --git https://github.com/IamShreshth/Tman.git --locked

# If using Zsh:
echo 'eval "$(tman init zsh)"' >> ~/.zshrc

# If using Bash (default on most Linux distros):
echo 'eval "$(tman init bash)"' >> ~/.bashrc
```

### Build from Source
```bash
git clone https://github.com/IamShreshth/Tman.git
cd Tman
./files/install.sh
```

### Run with Docker (Linux Container)
```bash
# Run unit tests inside a Linux container:
docker run --rm -v "$PWD":/usr/src/tman -w /usr/src/tman rust:slim cargo test

# Launch the interactive overlay inside Linux:
docker run --rm -it -v "$PWD":/usr/src/tman -w /usr/src/tman rust:slim cargo run -- ui
```

---

## Uninstallation

```bash
rm -f ~/.local/bin/tman ~/.cargo/bin/tman
rm -rf ~/.config/tman
```

Remove the `eval "$(tman init ...)"` line from your `~/.zshrc` or `~/.bashrc`.

---

## License

Licensed under the [MIT License](LICENSE).
