# ⚡ tman (terminalman)
> **A context-aware, hyper-fast interactive layer over your terminal.**

`tman` drops an interactive HUD right over your shell prompt with **Ctrl+Shift+T** (or **Ctrl+Space**), letting you navigate directories, run project workflows, inspect system vitals, search history, and manage bookmarks without leaving your active workflow.

---

## 🚀 Key Features

### 1. 🖥️ Mission Control HUD
When opened on the dashboard, the right-hand context panel displays real-time vitals:
- **Power & Battery**: Charge percentage & charging status (`pmset`).
- **Uptime**: System boot time formatted cleanly.
- **CPU Load**: 1m, 5m, 15m load averages (`getloadavg`).
- **RAM**: Total system memory (`sysctl`).
- **Disk Space**: Free storage space and percentage used on current volume (`statvfs`).
- **Git Context**: Current branch, staged (+), modified (~), and untracked (?) files.

### 2. ★ Pinned Bookmarks
- Press **`b`** on any folder in Navigate / Files view to pin it to your quick bookmarks.
- Open **Bookmarks** from the main dashboard to jump directly into your favorite project folders from anywhere.
- Press **`d`** or **`x`** inside the Bookmarks view to remove a pin.
- All bookmarks are saved persistently in `~/.config/tman/config.toml`.

### 3. ▶️ Smarter Multi-Stack Project Runners
Inside any repository or project directory, open **Run** to see auto-discovered workflow commands:
- **Python**: Detects `.venv` / `venv` (`source .venv/bin/activate`), Poetry (`poetry run ...`), Pytest (`pytest`), Pip (`pip install -r requirements.txt`), and `main.py` / `app.py`.
- **Docker & Compose**: Detects `docker-compose.yml` / `compose.yaml` (`docker compose up -d`, `down`, `logs -f`, `restart`, `ps`) or `Dockerfile` (`docker build`).
- **Node.js**: Automatically identifies the right package manager (`bun`, `pnpm`, `yarn`, `npm`) and lists all scripts from `package.json`.
- **Rust**: `cargo build`, `cargo run`, `cargo test`, `cargo check`, `cargo clippy`.
- **Go**: `go run .`, `go test ./...`, `go build`, `go mod tidy`.
- **Make & Just**: Dynamically parses targets from `Makefile` and `justfile` / `Justfile`.

### 4. 🎨 In-App Theme Selector (like btop)
- Choose from 12 curated aesthetic color themes directly from the dashboard:
  `cyan` 💎, `blue` 🌊, `green` 🌿, `magenta` 🔮, `yellow` ☀️, `red` 🔥, `orange` 🍊, `purple` 👾, `pink` 🌸, `peach` 🍑, `teal` ✨, `white` ❄️, or custom `#hex`.
- Selection applies immediately with real-time palette preview and saves to your config file.

### 5. 🛡️ Safety & Frictionless Shell Execution
- **Real shell execution**: Running `tman` in your shell evaluates `cd` or your selected command directly in your active shell prompt.
- **Destructive action warning**: Destructive commands (like `rm -rf`, dropping tables, git resets) trigger a confirmation overlay with an option to press **`E`** to edit the command before running.

---

## 📦 Installation & Setup

### 1. Build the release binary
```bash
cargo build --release
```

### 2. Install to PATH
On macOS:
```bash
sudo rm -f /usr/local/bin/tman
sudo cp target/release/tman /usr/local/bin/tman
sudo codesign --force --deep --sign - /usr/local/bin/tman
```
*(Or install to `~/.local/bin/tman` if `~/.local/bin` is in your PATH)*

### 3. Enable Shell Integration
Add this line to your `~/.zshrc` (or `~/.bashrc`):
```bash
eval "$(tman init zsh)"
```
Then reload your shell:
```bash
source ~/.zshrc
```

---

## ⌨️ Keybindings

| Key | Action |
| :--- | :--- |
| **Ctrl+Shift+T** / **Ctrl+Space** | Open `tman` HUD over your shell prompt |
| **↑ / ↓** or **j / k** | Navigate up / down |
| **Enter** | Select / Run / cd into directory |
| **→** | Enter directory (in Navigate view) |
| **←** / **Backspace** | Go up to parent directory / back to previous menu |
| **b** | Bookmark / Pin selected directory |
| **d** or **x** | Delete bookmark (inside Bookmarks view) |
| **Tab** | Open File Actions menu / Insert command to edit |
| **/** | Fuzzy search items |
| **.** | Toggle hidden dotfiles on/off |
| **m** | Jump back to Main Menu |
| **Esc** | Close `tman` / Clear search |
| **Ctrl+C** / **Ctrl+D** | Exit immediately |

---

## ⚙️ Configuration

Configuration is located at `~/.config/tman/config.toml`:

```toml
[ui]
animations = true     # smooth slide-in context panel
preview = true        # enable context / preview panel
show_hidden = false
accent = "cyan"       # cyan | blue | green | magenta | yellow | red | orange | purple | pink | peach | teal | white | #hex

[keybindings]
open = "ctrl-shift-t" # keybinding handled by tman init

[shell]
editor = ""           # defaults to $VISUAL / $EDITOR / vi

[safety]
confirm_destructive = true
```

Generate the default config at any time:
```bash
tman default-config > "$(tman config-path)"
```
