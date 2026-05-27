#!/usr/bin/env bash
# =============================================================================
# DroidOps — Local Build Script (Linux / macOS)
# Usage:
#   ./scripts/build.sh              — build for current platform (all targets)
#   ./scripts/build.sh --deb        — Linux .deb only
#   ./scripts/build.sh --appimage   — Linux .AppImage only
#   ./scripts/build.sh --rpm        — Linux .rpm only
#   ./scripts/build.sh --dmg        — macOS .dmg (must run on macOS)
#   ./scripts/build.sh --universal  — macOS universal binary (Intel + Apple Silicon)
#   ./scripts/build.sh --debug      — debug build (faster, no optimizations)
# =============================================================================
set -euo pipefail

# ── Colors ──────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

info()    { echo -e "${CYAN}[build]${NC} $*"; }
success() { echo -e "${GREEN}[✓]${NC} $*"; }
warn()    { echo -e "${YELLOW}[!]${NC} $*"; }
error()   { echo -e "${RED}[✗]${NC} $*"; exit 1; }

# ── Root detection ───────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

# ── Read version from package.json ──────────────────────────────────────────
VERSION="$(node -p "require('./package.json').version" 2>/dev/null || echo "unknown")"

echo ""
echo -e "${BOLD}══════════════════════════════════════════${NC}"
echo -e "${BOLD}   DroidOps Build Script  v${VERSION}${NC}"
echo -e "${BOLD}══════════════════════════════════════════${NC}"
echo ""

# ── Parse arguments ──────────────────────────────────────────────────────────
TARGET_ARGS=""
DEBUG_BUILD=false
PLATFORM="$(uname -s)"

for arg in "$@"; do
  case $arg in
    --deb)       TARGET_ARGS="--bundles deb" ;;
    --appimage)  TARGET_ARGS="--bundles appimage" ;;
    --rpm)       TARGET_ARGS="--bundles rpm" ;;
    --dmg)       TARGET_ARGS="--bundles dmg" ;;
    --universal) TARGET_ARGS="--target universal-apple-darwin" ;;
    --debug)     DEBUG_BUILD=true ;;
    --help|-h)
      echo "Usage: $0 [--deb|--appimage|--rpm|--dmg|--universal|--debug]"
      exit 0 ;;
    *) warn "Unknown argument: $arg" ;;
  esac
done

# ── Prerequisite checks ──────────────────────────────────────────────────────
info "Checking prerequisites..."

command -v node  >/dev/null 2>&1 || error "Node.js not found. Install from https://nodejs.org"
command -v npm   >/dev/null 2>&1 || error "npm not found."
command -v cargo >/dev/null 2>&1 || error "Rust/Cargo not found. Install from https://rustup.rs"
command -v npx   >/dev/null 2>&1 || error "npx not found."

NODE_VER="$(node --version)"
RUST_VER="$(rustc --version 2>/dev/null | awk '{print $2}')"
success "Node.js ${NODE_VER}  |  Rust ${RUST_VER}"

# ── Linux-specific system deps ───────────────────────────────────────────────
if [[ "$PLATFORM" == "Linux" ]]; then
  info "Checking Linux system libraries..."
  MISSING_LIBS=()
  for lib in libwebkit2gtk-4.1-dev libsoup-3.0-dev librsvg2-dev; do
    dpkg -s "$lib" >/dev/null 2>&1 || MISSING_LIBS+=("$lib")
  done
  if [[ ${#MISSING_LIBS[@]} -gt 0 ]]; then
    warn "Missing system libraries: ${MISSING_LIBS[*]}"
    read -r -p "Install them now? [Y/n] " ans
    if [[ "$ans" =~ ^[Yy]?$ ]]; then
      sudo apt-get update
      sudo apt-get install -y "${MISSING_LIBS[@]}" \
        libayatana-appindicator3-dev patchelf libjavascriptcoregtk-4.1-dev libgtk-3-dev
    else
      error "Build cannot continue without system libraries."
    fi
  else
    success "All system libraries present."
  fi
fi

# ── Install npm deps ─────────────────────────────────────────────────────────
info "Installing npm dependencies..."
npm ci --prefer-offline 2>/dev/null || npm install
success "npm dependencies ready."

# ── Build ────────────────────────────────────────────────────────────────────
echo ""
if [[ "$DEBUG_BUILD" == true ]]; then
  info "Building DEBUG binary (fast, no optimizations)..."
  BUILD_CMD="npm run tauri build -- --debug ${TARGET_ARGS}"
else
  info "Building RELEASE binary for platform: ${PLATFORM}..."
  BUILD_CMD="npm run tauri build -- ${TARGET_ARGS}"
fi

info "Running: ${BUILD_CMD}"
echo ""
eval "$BUILD_CMD"

# ── Report artifacts ─────────────────────────────────────────────────────────
echo ""
success "Build complete!"
info "Artifacts located in:"

BUILD_DIR="src-tauri/target"
if [[ "$DEBUG_BUILD" == true ]]; then
  BUNDLE_DIR="${BUILD_DIR}/debug/bundle"
else
  BUNDLE_DIR="${BUILD_DIR}/release/bundle"
fi

if [[ -d "$BUNDLE_DIR" ]]; then
  find "$BUNDLE_DIR" -type f \( \
    -name "*.deb" -o -name "*.rpm" -o \
    -name "*.AppImage" -o -name "*.dmg" -o \
    -name "*.app.tar.gz" -o -name "*.msi" -o -name "*.exe" \
  \) | while read -r f; do
    SIZE="$(du -sh "$f" | cut -f1)"
    echo -e "  ${GREEN}→${NC} $f  (${SIZE})"
  done
else
  warn "Bundle directory not found: $BUNDLE_DIR"
fi

echo ""
