#!/usr/bin/env bash
# ==============================================================================
# tman (terminalman) Installer
# https://github.com/IamShreshth/Tman
# ==============================================================================
# Single-step install command:
#   curl -fsSL https://raw.githubusercontent.com/IamShreshth/Tman/main/install.sh | bash
# ==============================================================================

set -euo pipefail

# --- Color formatting ---
BOLD="$(tput bold 2>/dev/null || printf '')"
DIM="$(tput dim 2>/dev/null || printf '')"
CYAN="$(tput setaf 6 2>/dev/null || printf '')"
GREEN="$(tput setaf 2 2>/dev/null || printf '')"
YELLOW="$(tput setaf 3 2>/dev/null || printf '')"
RED="$(tput setaf 1 2>/dev/null || printf '')"
RESET="$(tput sgr0 2>/dev/null || printf '')"

info()    { printf "${CYAN}::${RESET} ${BOLD}%s${RESET}\n" "$*"; }
success() { printf "${GREEN}[OK]${RESET} ${BOLD}%s${RESET}\n" "$*"; }
warn()    { printf "${YELLOW}[WARN]${RESET} %s\n" "$*"; }
error()   { printf "${RED}[ERROR]${RESET} %s\n" "$*" >&2; exit 1; }

REPO_URL="https://github.com/IamShreshth/Tman"
REPO_GIT="https://github.com/IamShreshth/Tman.git"

printf "\n"
printf "${CYAN}${BOLD}"
cat << "EOF"
   __                                 
  / /_____ ___  ____ _____           
 / __/ __ `__ \/ __ `/ __ \          
/ /_/ / / / / / /_/ / / / /          
\__/_/ /_/ /_/\__,_/_/ /_/  HUD      
EOF
printf "${RESET}"
printf "${DIM}Context-aware interactive layer for your terminal${RESET}\n\n"

# --- Detect OS & Architecture ---
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin)
        OS_TYPE="apple-darwin"
        ;;
    Linux)
        OS_TYPE="unknown-linux-gnu"
        ;;
    *)
        error "Unsupported operating system: $OS. tman supports macOS and Linux."
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        ARCH_TYPE="x86_64"
        ;;
    arm64|aarch64)
        ARCH_TYPE="aarch64"
        ;;
    *)
        error "Unsupported architecture: $ARCH."
        ;;
esac

TARGET="${ARCH_TYPE}-${OS_TYPE}"
info "Detected platform: ${BOLD}${OS} (${ARCH})${RESET}"

# --- Determine Destination Directory ---
if [[ -n "${PREFIX:-}" ]]; then
    BIN_DIR="$PREFIX/bin"
elif [[ "$EUID" -eq 0 ]]; then
    BIN_DIR="/usr/local/bin"
else
    BIN_DIR="$HOME/.local/bin"
fi

mkdir -p "$BIN_DIR"

# --- Installation Strategy ---
# 1. Local checkout build (if installer run from inside clone)
# 2. Pre-built release binary from GitHub Releases (if published)
# 3. Cargo install from Git / local source

INSTALLED=false

if [[ -f "./Cargo.toml" ]] && grep -q 'name = "tman"' "./Cargo.toml" 2>/dev/null; then
    info "Found local tman source repository. Building release binary..."
    if ! command -v cargo >/dev/null 2>&1; then
        error "Cargo is required to build from source. Install Rust from https://rustup.rs"
    fi
    cargo build --release
    cp "target/release/tman" "$BIN_DIR/tman"
    INSTALLED=true
fi

if [[ "$INSTALLED" = false ]]; then
    # Try downloading from GitHub Releases
    RELEASE_URL="${REPO_URL}/releases/latest/download/tman-${TARGET}.tar.gz"
    TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t 'tman-install')"
    trap 'rm -rf "$TMP_DIR"' EXIT

    info "Checking for pre-compiled binary release..."
    HTTP_CODE=$(curl -sL -w "%{http_code}" -o "$TMP_DIR/tman.tar.gz" "$RELEASE_URL" 2>/dev/null || true)

    if [[ "$HTTP_CODE" == "200" ]] && tar -tzf "$TMP_DIR/tman.tar.gz" >/dev/null 2>&1; then
        info "Downloading pre-compiled binary for ${TARGET}..."
        tar -xzf "$TMP_DIR/tman.tar.gz" -C "$TMP_DIR"
        cp "$TMP_DIR/tman" "$BIN_DIR/tman"
        INSTALLED=true
    else
        info "No pre-built binary release found for ${TARGET}. Falling back to Cargo..."
        if command -v cargo >/dev/null 2>&1; then
            info "Building and installing tman with Cargo..."
            cargo install --git "$REPO_GIT" --bin tman --locked --root "${BIN_DIR}/.."
            if [[ -f "${BIN_DIR}/../bin/tman" ]] && [[ "${BIN_DIR}/../bin/tman" != "$BIN_DIR/tman" ]]; then
                cp "${BIN_DIR}/../bin/tman" "$BIN_DIR/tman"
            fi
            INSTALLED=true
        else
            printf "\n"
            warn "Cargo (Rust toolchain) was not found."
            echo "To install Rust and Cargo, run:"
            echo "    ${CYAN}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${RESET}"
            echo ""
            error "Please install Rust or download a pre-built release binary from: ${REPO_URL}/releases"
        fi
    fi
fi

chmod +x "$BIN_DIR/tman"

# Codesign on macOS to prevent security prompts
if [[ "$OS" == "Darwin" ]]; then
    if command -v codesign >/dev/null 2>&1; then
        codesign --force --deep --sign - "$BIN_DIR/tman" 2>/dev/null || true
    fi
fi

success "Installed tman executable to: ${BOLD}$BIN_DIR/tman${RESET}"

# --- PATH Verification ---
PATH_NEEDS_UPDATE=false
case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) PATH_NEEDS_UPDATE=true ;;
esac

# --- Shell Integration Setup ---
CURRENT_SHELL="$(basename "${SHELL:-/bin/bash}")"
case "$CURRENT_SHELL" in
    zsh)
        RC_FILE="$HOME/.zshrc"
        SHELL_KIND="zsh"
        ;;
    bash)
        if [[ "$OS" == "Darwin" && -f "$HOME/.bash_profile" ]]; then
            RC_FILE="$HOME/.bash_profile"
        else
            RC_FILE="$HOME/.bashrc"
        fi
        SHELL_KIND="bash"
        ;;
    *)
        RC_FILE=""
        SHELL_KIND="zsh"
        ;;
esac

if [[ -n "$RC_FILE" ]]; then
    touch "$RC_FILE"
    
    # Check if PATH export is needed
    if [[ "$PATH_NEEDS_UPDATE" = true ]]; then
        if ! grep -q "$BIN_DIR" "$RC_FILE" 2>/dev/null; then
            printf '\n# Added by tman installer\nexport PATH="%s:$PATH"\n' "$BIN_DIR" >> "$RC_FILE"
            info "Added ${BIN_DIR} to PATH in ${RC_FILE}"
        fi
    fi

    # Check if shell integration is already installed
    if ! grep -q 'tman init' "$RC_FILE" 2>/dev/null; then
        printf '\n# tman interactive HUD integration\nif command -v tman >/dev/null 2>&1; then\n  eval "$(tman init %s)"\nfi\n' "$SHELL_KIND" >> "$RC_FILE"
        success "Added shell integration to ${BOLD}${RC_FILE}${RESET}"
    else
        success "Shell integration already present in ${BOLD}${RC_FILE}${RESET}"
    fi
fi

printf "\n"
printf "${GREEN}${BOLD}Installation Complete.${RESET}\n\n"
echo "To activate tman immediately in your current terminal:"
if [[ -n "$RC_FILE" ]]; then
    printf "  ${CYAN}source %s${RESET}\n\n" "$RC_FILE"
else
    printf "  ${CYAN}eval \"\$(tman init zsh)\"${RESET}\n\n"
fi

echo "Usage:"
echo "  - Press Ctrl+Shift+T (or Ctrl+Space) anywhere in your shell to toggle the HUD."
echo "  - Run 'tman' directly from any folder."
echo "  - Run 'tman --help' to view available options."
printf "\n"
