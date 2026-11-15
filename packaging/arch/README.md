# Arch Linux package

This PKGBUILD builds the native egui application, installs `/usr/bin/droidops`,
and registers a DroidOps application-menu entry with scalable and PNG icons.
It does not build or install the historical Tauri/React frontend.

## Build and install

On Arch Linux with base-devel installed:

```bash
cd packaging/arch
makepkg -si
```

makepkg installs missing build/runtime dependencies through pacman with normal
authorization. Rust/Cargo, librsvg, and desktop-file-utils are build dependencies.
ADB/Fastboot come from android-tools; scrcpy is optional for mirroring. Native
file dialogs require xdg-desktop-portal and an appropriate desktop portal backend.
The package never installs USB rules or changes Android devices.

By default Cargo uses at most two build jobs to limit peak build memory. Override
with `CARGO_BUILD_JOBS` when appropriate. Cargo dependencies are fetched in
prepare(), then builds/tests run frozen/offline against Cargo.lock.

For separate build and install steps:

```bash
makepkg
sudo pacman -U ./droidops-0.1.2-1-x86_64.pkg.tar.zst
```

Launch DroidOps from the application menu or run `droidops`. Remove it with
`sudo pacman -R droidops`. Package files belong to pacman; do not copy them into
system directories manually.

## Source and version

The package pins public commit `08414ac823609d544876bd59621bac31a9462f79`, the
native application currently on dev. Version 0.1.2 matches workspace metadata;
it is a development snapshot, not a new upstream release or a reset of tags.
Only x86_64 is declared. Other architectures need their own build/acceptance evidence.

When updating, select the reviewed source commit, update `_commit` and pkgver
from Cargo workspace metadata, increment pkgrel when only packaging changes,
and recalculate all SHA-256 checksums after inspecting the new sources. Validate
the desktop file, package contents/dependencies, and launch before installation.

The project license remains undeclared. The package uses LicenseRef-Unknown and
ships LICENSE_STATUS rather than inventing license text or ownership. No AUR
submission or release publication is part of this local packaging workflow.
