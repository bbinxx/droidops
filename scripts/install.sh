#!/usr/bin/env bash
# =============================================================================
# DroidOps — End-User Install Script
# Installs the latest DroidOps release for your platform from GitHub.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/bbinxx/ADBplus/main/scripts/install.sh | bash
#   -- OR --
#   ./scripts/install.sh
#   ./scripts/install.sh --version v0.1.5
#   ./scripts/install.sh --format appimage
#   ./scripts/install.sh --format deb
# =============================================================================
set -euo pipefail

# ── Config ───────────────────────────────────────────────────────────────────
REPO="bbinxx/ADBplus"
APP_NAME="DroidOps"
INSTALL_DIR_LINUX="/usr/local/bin"
APPIMAGE_INSTALL_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
GITHUB_API="https://api.github.com/repos/${REPO}/releases"

# ── Colors ───────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

info()    { echo -e "${CYAN}[install]${NC} $*"; }
success() { echo -e "${GREEN}[✓]${NC} $*"; }
warn()    { echo -e "${YELLOW}[!]${NC} $*"; }
error()   { echo -e "${RED}[✗]${NC} $*"; exit 1; }
step()    { echo -e "\n${BOLD}── $* ──${NC}"; }

# ── Parse args ───────────────────────────────────────────────────────────────
VERSION=""
FORMAT=""  # deb | appimage | rpm

while [[ $# -gt 0 ]]; do
  case $1 in
    --version) VERSION="$2"; shift 2 ;;
    --format)  FORMAT="$2";  shift 2 ;;
    --help|-h)
      echo "Usage: $0 [--version v0.1.5] [--format deb|appimage|rpm]"
      exit 0 ;;
    *) warn "Unknown arg: $1"; shift ;;
  esac
done

# ── Header ───────────────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}╔══════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║   ${APP_NAME} Installer                     ║${NC}"
echo -e "${BOLD}║   github.com/${REPO}        ║${NC}"
echo -e "${BOLD}╚══════════════════════════════════════════╝${NC}"
echo ""

# ── Platform detect ──────────────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

if [[ "$OS" != "Linux" && "$OS" != "Darwin" ]]; then
  error "This installer supports Linux and macOS. For Windows, download the .msi from GitHub Releases."
fi

info "Detected: ${OS} / ${ARCH}"

# ── Dependency check ─────────────────────────────────────────────────────────
step "Checking dependencies"

for dep in curl jq; do
  if ! command -v "$dep" >/dev/null 2>&1; then
    warn "$dep is not installed. Attempting to install..."
    if [[ "$OS" == "Linux" ]]; then
      sudo apt-get install -y "$dep" 2>/dev/null || sudo dnf install -y "$dep" 2>/dev/null || \
        error "Could not install $dep. Please install it manually."
    elif [[ "$OS" == "Darwin" ]]; then
      brew install "$dep" 2>/dev/null || error "Could not install $dep. Install Homebrew first."
    fi
  fi
done
success "All dependencies present."

# ── Fetch latest release info ────────────────────────────────────────────────
step "Fetching release information"

if [[ -n "$VERSION" ]]; then
  info "Fetching specific version: $VERSION"
  RELEASE_JSON="$(curl -fsSL "${GITHUB_API}/tags/${VERSION}")"
else
  info "Fetching latest release..."
  RELEASE_JSON="$(curl -fsSL "${GITHUB_API}/latest")"
fi

RELEASE_TAG="$(echo "$RELEASE_JSON" | jq -r '.tag_name')"
RELEASE_NAME="$(echo "$RELEASE_JSON" | jq -r '.name')"

if [[ "$RELEASE_TAG" == "null" || -z "$RELEASE_TAG" ]]; then
  error "Could not fetch release info from GitHub. Check your internet connection or https://github.com/${REPO}/releases"
fi

success "Found: ${RELEASE_NAME} (${RELEASE_TAG})"

# ── Select asset ─────────────────────────────────────────────────────────────
step "Selecting installer"

# List all assets
ASSETS="$(echo "$RELEASE_JSON" | jq -r '.assets[] | "\(.name)|\(.browser_download_url)"')"

pick_asset() {
  local pattern="$1"
  echo "$ASSETS" | grep -i "$pattern" | head -1 | cut -d'|' -f2
}

if [[ "$OS" == "Darwin" ]]; then
  if [[ "$ARCH" == "arm64" ]]; then
    ASSET_URL="$(pick_asset 'universal.*dmg\|aarch64.*dmg')"
    [[ -z "$ASSET_URL" ]] && ASSET_URL="$(pick_asset '\.dmg$')"
  else
    ASSET_URL="$(pick_asset 'universal.*dmg\|x86_64.*dmg')"
    [[ -z "$ASSET_URL" ]] && ASSET_URL="$(pick_asset '\.dmg$')"
  fi
  FORMAT_LABEL="macOS DMG"
elif [[ "$OS" == "Linux" ]]; then
  if [[ "$FORMAT" == "rpm" ]]; then
    ASSET_URL="$(pick_asset '\.rpm$')"
    FORMAT_LABEL="RPM package"
  elif [[ "$FORMAT" == "deb" ]] || dpkg --version >/dev/null 2>&1; then
    ASSET_URL="$(pick_asset '\.deb$')"
    FORMAT_LABEL="Debian package (.deb)"
  else
    ASSET_URL="$(pick_asset '\.AppImage$')"
    FORMAT_LABEL="AppImage"
  fi
fi

if [[ -z "$ASSET_URL" ]]; then
  error "No matching installer found for ${OS}/${ARCH}. Available assets:\n$(echo "$ASSETS" | cut -d'|' -f1)"
fi

FILENAME="$(basename "$ASSET_URL")"
info "Selected: ${FILENAME}  (${FORMAT_LABEL})"

# ── Download ─────────────────────────────────────────────────────────────────
step "Downloading"

TMPDIR="$(mktemp -d)"
DEST="${TMPDIR}/${FILENAME}"

info "Downloading from GitHub..."
curl -L --progress-bar -o "$DEST" "$ASSET_URL"
success "Downloaded: ${DEST} ($(du -sh "$DEST" | cut -f1))"

# ── Install ──────────────────────────────────────────────────────────────────
step "Installing"

if [[ "$OS" == "Darwin" ]]; then
  # macOS: mount the DMG and copy the .app
  info "Mounting disk image..."
  MOUNT_POINT="$(hdiutil attach "$DEST" -nobrowse -readonly | grep 'Volumes' | awk '{print $NF}')"
  APP_SRC="$(ls "$MOUNT_POINT"/*.app 2>/dev/null | head -1)"
  if [[ -z "$APP_SRC" ]]; then
    error "No .app found in disk image at $MOUNT_POINT"
  fi
  APP_DEST="/Applications/$(basename "$APP_SRC")"
  info "Copying ${APP_SRC} → ${APP_DEST}"
  cp -Rf "$APP_SRC" "$APP_DEST"
  hdiutil detach "$MOUNT_POINT" -quiet
  success "${APP_NAME} installed to ${APP_DEST}"

elif [[ "$FILENAME" == *.deb ]]; then
  info "Installing .deb package (requires sudo)..."
  sudo dpkg -i "$DEST"
  success "${APP_NAME} installed via dpkg."

elif [[ "$FILENAME" == *.rpm ]]; then
  info "Installing .rpm package (requires sudo)..."
  if command -v dnf >/dev/null 2>&1; then
    sudo dnf install -y "$DEST"
  else
    sudo rpm -i --force "$DEST"
  fi
  success "${APP_NAME} installed via rpm."

elif [[ "$FILENAME" == *.AppImage ]]; then
  mkdir -p "$APPIMAGE_INSTALL_DIR"
  APPIMAGE_DEST="${APPIMAGE_INSTALL_DIR}/DroidOps.AppImage"
  cp "$DEST" "$APPIMAGE_DEST"
  chmod +x "$APPIMAGE_DEST"

  # Create symlink in PATH
  if [[ -w "$INSTALL_DIR_LINUX" ]]; then
    ln -sf "$APPIMAGE_DEST" "${INSTALL_DIR_LINUX}/droidops"
  else
    sudo ln -sf "$APPIMAGE_DEST" "${INSTALL_DIR_LINUX}/droidops"
  fi

  # Create .desktop launcher
  mkdir -p "$DESKTOP_DIR"
  cat > "${DESKTOP_DIR}/droidops.desktop" << EOF
[Desktop Entry]
Name=DroidOps
Comment=Android Device Manager
Exec=${APPIMAGE_DEST}
Icon=droidops
Terminal=false
Type=Application
Categories=Utility;Development;
StartupWMClass=DroidOps
EOF
  success "AppImage installed to ${APPIMAGE_DEST}"
  success "Desktop launcher created at ${DESKTOP_DIR}/droidops.desktop"
  info "Run with: droidops"
fi

# ── ADB check ────────────────────────────────────────────────────────────────
step "Checking ADB (required for device communication)"

if command -v adb >/dev/null 2>&1; then
  ADB_VER="$(adb version | head -1)"
  success "ADB found: ${ADB_VER}"
else
  warn "ADB not found in PATH."
  echo -e "  Install with:"
  if [[ "$OS" == "Linux" ]]; then
    echo -e "    ${CYAN}sudo apt-get install android-tools-adb android-tools-fastboot${NC}"
  elif [[ "$OS" == "Darwin" ]]; then
    echo -e "    ${CYAN}brew install android-platform-tools${NC}"
  fi
fi

# ── Cleanup ───────────────────────────────────────────────────────────────────
rm -rf "$TMPDIR"

# ── Done ─────────────────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}${GREEN}╔══════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${GREEN}║   ${APP_NAME} ${RELEASE_TAG} installed!${NC}"
echo -e "${BOLD}${GREEN}╚══════════════════════════════════════════╝${NC}"
echo ""
info "Connect your Android device with USB Debugging enabled and launch DroidOps."
echo ""
