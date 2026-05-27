#!/usr/bin/env bash

# Exit immediately if a command exits with a non-zero status
set -e

echo "========================================="
echo "   DroidOps: Upgrading Scrcpy for Android 16"
echo "========================================="
echo ""

# 1. Install build dependencies
echo "[1/4] Installing compilation dependencies..."
sudo apt-get update
sudo apt-get install -y \
    ffmpeg \
    libsdl2-2.0-0 \
    libusb-1.0-0 \
    wget \
    gcc \
    git \
    pkg-config \
    meson \
    ninja-build \
    libsdl2-dev \
    libavcodec-dev \
    libavdevice-dev \
    libavformat-dev \
    libavutil-dev \
    libswresample-dev \
    libusb-1.0-0-dev \
    libv4l-dev

# 2. Setup temp directory and clone
TEMP_DIR=$(mktemp -d)
echo "[2/4] Cloning official Scrcpy source into $TEMP_DIR..."
git clone --depth 1 https://github.com/Genymobile/scrcpy.git "$TEMP_DIR/scrcpy"

# 3. Build and install release
echo "[3/4] Compiling and installing the latest release..."
cd "$TEMP_DIR/scrcpy"
./install_release.sh

# 4. Clean up
echo "[4/4] Cleaning up temporary build artifacts..."
rm -rf "$TEMP_DIR"

echo ""
echo "========================================="
echo "   SUCCESS: Scrcpy upgraded to the latest version!"
echo "   You can verify with: scrcpy --version"
echo "========================================="
