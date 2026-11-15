# Feature parity

States describe the native implementation. Implemented means code exists; Tested
means automated coverage exists for the named part; Verified requires recorded
manual acceptance for that workflow/platform. Partial limitations are explicit.

| Feature | Native state | Evidence / remaining work |
| --- | --- | --- |
| ADB/Fastboot discovery | Tested | Parser fixtures; Linux live discovery; hardware reconnection acceptance pending |
| Explicit device selection | Tested | No auto-selection, disconnect/state invalidation, stale generation tests |
| Dashboard | Implemented | Rust property/battery queries and native UI; hardware details not manually verified |
| Applications | Tested | User/system/all package inventory and mock parsing; scrcpy labels with package-ID fallback; generic vector icons, APK icon extraction remains Planned |
| APK installation | Implemented | Picker/drop and replacement confirmation; no physical installation test |
| Uninstall / clear app data | Tested | Confirmation and Android failure marker tests; hardware acceptance pending |
| File listing | Tested | NUL-framed stat records, direct-child membership, malformed/newline fixture tests |
| File upload/download | Implemented | Native dialogs/drop, overwrite disclosure, timeout/cancellation; hardware transfer acceptance pending |
| File create/rename/delete | Tested | Command policy and disposable host symlink/collision tests; Android Toybox compatibility needs acceptance |
| File/image thumbnails | Planned | Legacy full-file cache is not copied; no persistent thumbnail cache yet |
| Gallery | Implemented | DCIM/Pictures search, album filter, bounded preview decoding, downloads; other media locations/formats pending |
| One-shot ADB shell | Implemented | Raw command confirmation and bounded output; no persistent terminal |
| Recovery ZIP sideload | Implemented | Distinct screen, ZIP picker, sideload-state gate and confirmation; hardware acceptance pending |
| Power / Fastboot reboot | Tested | Target/mode command tests; no flashing/unlocking surface |
| Screen mirroring | Implemented | Independent cancellable scrcpy task; real mirroring acceptance pending |
| Independent app windows | Tested | Native owned scrcpy sessions, virtual-display launch verification, configurable settings, per-session Stop and shutdown; [evidence and limitations](../features/ANDROID_APP_WINDOWS.md) |
| Tool installation | Planned | Native guidance replaces Linux-specific elevated installation; no silent elevation |
| Settings | Implemented | Saved light/dark theme and interface scale; session/device data excluded |
| Native screen layout | Tested | Fixture rendering at two scales and vector-icon tests; Linux live capture |
| Windows/macOS acceptance | Not started | Native CI matrix added; no claim of completed runner or hardware checks |
| Native installers/releases | Planned | Existing Tauri release workflow retained; native publishing not configured |

Legacy removal remains gated on review, feature acceptance, and explicit approval.
