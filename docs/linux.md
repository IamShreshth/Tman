# Linux Guide for tman

`tman` runs natively on Linux (both `x86_64` and `aarch64` architectures) with zero external runtime dependencies.

---

## 1. Installation

### Quick Install (Single Command)
```bash
curl -fsSL https://raw.githubusercontent.com/IamShreshth/Tman/main/files/install.sh | bash
```

The installer:
- Auto-detects your architecture (`x86_64` or `aarch64`) and libc (glibc or musl for Alpine).
- Places the standalone binary in `~/.local/bin/tman` (or `~/.cargo/bin`).
- Adds the appropriate shell hook to your `~/.bashrc` or `~/.zshrc`.

### Via Cargo
```bash
cargo install --git https://github.com/IamShreshth/Tman.git --locked
```

Then append the shell hook:
- **Bash**:
  ```bash
  echo 'eval "$(tman init bash)"' >> ~/.bashrc
  source ~/.bashrc
  ```
- **Zsh**:
  ```bash
  echo 'eval "$(tman init zsh)"' >> ~/.zshrc
  source ~/.zshrc
  ```

---

## 2. Launching & Keyboard Shortcuts

By default, `tman init` configures **both** `Ctrl+Shift+T` and `Ctrl+Space`:

| Key | Notes |
| :--- | :--- |
| **`Ctrl+Space`** | **Recommended on Linux**: Terminal emulators (e.g., GNOME Terminal, Tilix, XFCE Terminal) reserve `Ctrl+Shift+T` for "Open New Tab". `Ctrl+Space` bypasses this conflict cleanly. |
| **`Ctrl+Shift+T`** | Works out of the box in terminals with CSI u support (Kitty, Foot, WezTerm, Alacritty) or if "New Tab" is remapped. |
| **`tman`** | Run `tman` directly in your shell anytime. |

---

## 3. System Telemetry & Vitals

On Linux, `tman` queries the Linux kernel and virtual filesystems directly:

* **Power / Battery**: Queries `/sys/class/power_supply/BAT*` for percentage and charge status. Gracefully hidden on desktops/servers without a battery.
* **Memory (RAM)**: Reads total memory from `/proc/meminfo` with fallback to the `sysinfo(2)` syscall.
* **CPU Load**: Queries `libc::getloadavg` with fallback to `/proc/loadavg`.
* **Uptime**: Reads `/proc/uptime` with fallback to `sysinfo(2)`.
* **Disk Space**: Uses standard POSIX `statvfs(2)` with fragment and block size fallbacks.

---

## 4. Clipboard & Desktop Integration

* **Clipboard**:
  * **Wayland**: Install `wl-clipboard` (`sudo apt install wl-clipboard` / `sudo pacman -S wl-clipboard`).
  * **X11**: Install `xclip` (`sudo apt install xclip`) or `xsel`.
* **System Opener**:
  * Pressing `Tab` on a file provides an **"Open with app"** action (`xdg-open`).
  * Pressing `Tab` on a folder provides an **"Open file manager"** action (`xdg-open`).

---

## 5. Testing with Docker

To test `tman` inside a clean Linux container without affecting your host:

```bash
# Run unit tests inside Linux
docker run --rm -v "$PWD":/usr/src/tman -w /usr/src/tman rust:slim cargo test

# Launch the interactive HUD inside Linux
docker run --rm -it -v "$PWD":/usr/src/tman -w /usr/src/tman rust:slim cargo run -- ui
```
