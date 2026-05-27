# DroidOps — Release Notes

> A native Tauri desktop application for Android device management via ADB.
> Built with React + Vite + Rust (Tauri v2).

---

## v0.1.5 — `2026-05-27`
**Screen Mirroring, Per-App Cast & ADB Auto-Installer**

### ✨ New Features
- **Screen Mirroring** — Mirror button in the device selector bar launches `scrcpy` as a detached native window, with `--stay-awake` and `--always-on-top` enabled by default.
- **Per-App Cast** — Hover over any app row in the App Manager to reveal a violet Cast icon. Clicking it uses `adb shell monkey` to bring that app to the foreground, then opens scrcpy mirroring automatically.
- **ADB Auto-Installer** — Dashboard diagnostics now show an **Auto-Install ADB & Fastboot** button when `adb` is not found on a Tauri desktop session. Uses `pkexec apt-get` to install without blocking the UI.
- **InstallModal** — New glassmorphic progress modal streams real-time installation output, parsing `apt-get` stdout into granular progress percentages (0–100%).
- **`upgrade_scrcpy.sh`** — Automated shell script to compile and install `scrcpy v2.7` from source on Ubuntu/Linux Mint. Handles all `SDL2`-based build dependencies automatically.

### 🐛 Bug Fixes
- Fixed JSX "adjacent elements must be wrapped" error in `Dashboard.jsx` (InstallModal placed outside root div).
- Fixed Mirror button failing silently due to Tauri capability `cmd` pointing at old `scrcpy v1.25` binary (`/usr/bin/scrcpy`) instead of newly compiled `v2.7` (`/usr/local/bin/scrcpy`). Capabilities now use the absolute path.
- Fixed `--bit-rate` flag renamed to `-b` in `scrcpy v2.x`.
- Removed unsupported `--start-app` flag (not available in v2.7). Replaced with `adb shell monkey -p <package>` pre-launch step.

### 🛠 Improvements
- Full `stdout` / `stderr` listeners added to all scrcpy spawn calls — errors are now surfaced to the user as alerts with detailed messages.
- Added `[scrcpy]` and `[scrcpy-app]` prefixed console logs throughout the mirroring pipeline for easy DevTools debugging.
- Dashboard diagnostics: each status card now conditionally renders an action button (install / retry) based on detected state.
- DeviceSelector styled with premium glassmorphic dark theme (`#16161a`, `#0f0f12`).

### 📦 Files Changed
| File | Change |
|---|---|
| `src/lib/adb.js` | Added `runScrcpyApp`, rewrote `runScrcpy` with debug logging, added `checkScrcpyInstalled`, `installDependencies` |
| `src/components/DeviceSelector.jsx` | Mirror button wired to `runScrcpy`, scrcpy check + install flow |
| `src/components/InstallModal.jsx` | New component — real-time install progress modal |
| `src/pages/AppManager.jsx` | Per-app Cast button with `runScrcpyApp` |
| `src/pages/Dashboard.jsx` | Auto-install ADB button, InstallModal integration, state hooks |
| `src-tauri/capabilities/default.json` | Absolute path `/usr/local/bin/scrcpy` in capabilities |
| `upgrade_scrcpy.sh` | New script to build scrcpy v2.7 from source |

---

## v0.1.4 — `2025-12-25`
**Real App Icons with Caching & Lazy Loading**

### ✨ New Features
- Real application icons fetched from device via `adb` and rendered per-app row in the App Manager.
- Icon caching layer — icons are pulled once and stored in memory to avoid redundant ADB calls.
- Lazy loading pattern: icons load progressively as they scroll into view, preventing UI freeze on large app lists.

### 🛠 Improvements
- Optimized app label fetching with batch processing — significantly reduced load time for `system` and `all` package views.
- `AppIcon` component created and extracted into its own file for reuse.

---

## v0.1.3 — `2025-12-25`
**Modular Architecture Refactor**

> *"The app is now fully modular and data-driven"*

### 🏗 Architecture
- Full refactor to modular component structure — pages, components, lib, utils, data, and hooks separated into dedicated directories.
- `STRINGS` data object introduced in `src/data/strings.js` — all UI text is now centrally managed and data-driven.
- `confirmUninstall`, `confirmClearData`, `alertDialog` helper utilities extracted to `src/utils/dialog.js`.
- `ARCHITECTURE.md` merged into `README.md` for single-source documentation.

---

## v0.1.2 — `2025-12-25`
**Gallery & Thumbnail Caching**

### ✨ New Features
- **Gallery view** — Browse and display images stored on the connected device.
- **Thumbnail caching** — Thumbnails pulled via `adb pull` are cached on disk to avoid re-fetching on subsequent opens.
- `Gallery` component added with a responsive grid layout.

### 🛠 Improvements
- `Sidebar` navigation component added for multi-page routing inside the app.

---

## v0.1.1 — `2025-12-23`
**CI: Multi-Platform Release Pipeline**

### 🔧 CI / Infrastructure
- GitHub Actions workflow added for automated multi-platform Tauri builds.
- Releases target: **Linux (AppImage + deb)**, **macOS (dmg)**, **Windows (msi + exe)**.
- Auto-changelog generation wired into the release pipeline.
- `dev → main` auto-merge workflow triggered on tag push.

---

## v0.1.0 — `2025-12-23`
**Initial Release**

### 🎉 Features
- Native Tauri v2 desktop shell with React + Vite frontend.
- ADB device detection and real-time tracking via `adb devices`.
- **App Manager** — list, uninstall, and clear data for user/system/all packages.
- **File Explorer** — browse, download, and upload files from device storage.
- **Shell Terminal** — interactive `adb shell` session inside the app.
- **Sideload** — drag-and-drop APK installation via `adb install`.
- **Fastboot support** — basic detection and mode awareness for bootloader devices.
- Dark-mode-first UI with DroidOps brand orange (`#E95420`) accent color.
- Cross-platform: Linux, macOS, Windows.

---

*For the latest source, issues, and contributions see [github.com/bbinxx/ADBplus](https://github.com/bbinxx/ADBplus)*
