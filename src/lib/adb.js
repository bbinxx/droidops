import { Command } from "@tauri-apps/plugin-shell";

/**
 * Executes an ADB command
 * @param {string[]} args 
 * @returns {Promise<string>} stdout
 */
export const runAdbCommand = async (args) => {
    try {
        const command = Command.create("adb", args);
        const output = await command.execute();
        if (output.code !== 0) {
            console.error("ADB Error:", output.stderr);
            throw new Error(output.stderr || "Unknown ADB Error");
        }
        return output.stdout;
    } catch (err) {
        console.error("Command Execution Failed:", err);
        throw err;
    }
};

/**
 * Executes a Fastboot command
 * @param {string[]} args 
 * @returns {Promise<string>}
 */
export const runFastbootCommand = async (args) => {
    try {
        const command = Command.create("fastboot", args);
        const output = await command.execute();
        if (output.code !== 0) {
            throw new Error(output.stderr || "Unknown Fastboot Error");
        }
        return output.stdout;
    } catch (err) {
        console.error("Fastboot Execution Failed:", err);
        throw err;
    }
};

/**
 * Launch Scrcpy for screen mirroring
 * @param {string} serial - Device serial number
 * @param {Object} options - Scrcpy options
 * @returns {Promise<void>}
 */
export const runScrcpy = async (serial, options = {}) => {
    try {
        const args = ["-s", serial];

        // Add optional parameters
        if (options.maxSize) args.push("--max-size", options.maxSize.toString());
        if (options.bitRate) args.push("--bit-rate", options.bitRate);
        if (options.maxFps) args.push("--max-fps", options.maxFps.toString());
        if (options.fullscreen) args.push("--fullscreen");
        if (options.alwaysOnTop) args.push("--always-on-top");
        if (options.turnScreenOff) args.push("--turn-screen-off");
        if (options.stayAwake) args.push("--stay-awake");
        if (options.noControl) args.push("--no-control");

        console.log("Launching scrcpy with args:", args);

        const command = Command.create("scrcpy", args);

        // Spawn scrcpy as a background process (don't wait for completion)
        await command.spawn();

        console.log("Scrcpy launched successfully");
    } catch (err) {
        console.error("Scrcpy execution failed:", err);

        // Better error message formatting
        const errorMsg = err?.message || err?.toString() || "Unknown error occurred";

        if (errorMsg.includes("not found") || errorMsg.includes("not recognized") || errorMsg.includes("program not found")) {
            throw new Error("Scrcpy not found. Please install scrcpy and ensure it's in your system PATH.\n\nInstall: https://github.com/Genymobile/scrcpy");
        }

        throw new Error(`Failed to launch scrcpy: ${errorMsg}`);
    }
};


export const getConnectedDevices = async () => {
    try {
        const stdout = await runAdbCommand(["devices", "-l"]);
        const lines = stdout.split("\n").filter(l => l.trim() && !l.startsWith("List of devices"));

        // Parse "serial device product:x model:y device:z transport_id:t"
        return lines.map(line => {
            const parts = line.split(/\s+/);
            const serial = parts[0];
            const state = parts[1]; // device, offline, unauthorized

            const modelPart = parts.find(p => p.startsWith("model:")) || "";
            const model = modelPart.split(":")[1] || "Unknown";

            return { serial, state, model, type: 'adb' };
        });
    } catch (e) {
        return [];
    }
};

export const getFastbootDevices = async () => {
    try {
        const stdout = await runFastbootCommand(["devices"]);
        const lines = stdout.split("\n").filter(l => l.trim());
        return lines.map(line => {
            const parts = line.split(/\s+/);
            return { serial: parts[0], state: 'fastboot', model: 'Fastboot Device', type: 'fastboot' };
        });
    } catch (e) {
        return [];
    }
}

export const getDeviceInfo = async (serial) => {
    if (!serial) return null;
    try {
        const [model, androidVer, batteryOut] = await Promise.all([
            runAdbCommand(["-s", serial, "shell", "getprop", "ro.product.model"]),
            runAdbCommand(["-s", serial, "shell", "getprop", "ro.build.version.release"]),
            runAdbCommand(["-s", serial, "shell", "dumpsys", "battery"])
        ]);

        // Parse Battery
        let level = "N/A";
        const levelMatch = batteryOut.match(/level: (\d+)/);
        if (levelMatch) level = levelMatch[1];

        let status = "Unknown";
        // 2: Charging, 3: Discharging, 4: Not charging, 5: Full
        const statusMatch = batteryOut.match(/status: (\d+)/);
        if (statusMatch) {
            const s = parseInt(statusMatch[1]);
            if (s === 2) status = "Charging";
            else if (s === 3) status = "Discharging";
            else if (s === 4) status = "Not Charging";
            else if (s === 5) status = "Full";
        }

        return {
            model: model.trim(),
            androidVersion: androidVer.trim(),
            batteryLevel: level,
            batteryStatus: status
        };
    } catch (e) {
        console.error("Failed to get device info", e);
        return null;
    }
};

export const getInstalledPackages = async (serial, type = "user") => {
    // type: 'user' (-3), 'system' (-s), 'all' (no flag)
    const args = ["-s", serial, "shell", "pm", "list", "packages"];
    if (type === 'user') args.push("-3");
    else if (type === 'system') args.push("-s");

    try {
        // Step 1: Get all package names (fast)
        const stdout = await runAdbCommand(args);
        const packages = stdout.split("\n")
            .filter(l => l.startsWith("package:"))
            .map(l => l.replace("package:", "").trim())
            .filter(p => p);

        if (packages.length === 0) return [];

        // Step 2: Get ALL app labels AND icon paths in batch (much faster!)
        const { labelMap, iconMap } = await getAllAppInfo(serial, packages);

        // Step 3: Combine package names with labels and icons
        const appsWithNames = packages.map(pkg => {
            const label = labelMap[pkg];
            const iconPath = iconMap[pkg];
            return {
                packageName: pkg,
                appName: label || pkg.split('.').pop(),
                displayName: label || pkg.split('.').pop(),
                iconPath: iconPath || null // Path to icon on device
            };
        });

        return appsWithNames.sort((a, b) => a.appName.localeCompare(b.appName));
    } catch (e) {
        console.error("Failed to get packages:", e);
        return [];
    }
};

/**
 * Get ALL app info (labels + icon paths) at once (MUCH FASTER than individual calls!)
 * This makes minimal commands instead of N commands for N packages
 */
const getAllAppInfo = async (serial, packages) => {
    try {
        const labelMap = {};
        const iconMap = {};

        // Batch approach: Get APK paths first
        const pathOutput = await runAdbCommand([
            "-s", serial, "shell", "pm", "list", "packages", "-f"
        ]);

        const apkPaths = {};
        pathOutput.split("\n").forEach(line => {
            // Format: package:/path/to/app.apk=com.example.app
            const match = line.match(/package:(.+\.apk)=(.+)/);
            if (match) {
                apkPaths[match[2].trim()] = match[1];
            }
        });

        // Extract labels and icon paths using dumpsys package (faster than aapt per-package)
        try {
            // Get full dumpsys package output ONCE
            const dumpsysOut = await runAdbCommand([
                "-s", serial, "shell", "dumpsys", "package", "packages"
            ]);

            // Parse the output to extract labels and resources
            packages.forEach(pkg => {
                // Find this package's section in dumpsys
                const pkgSection = extractPackageSection(dumpsysOut, pkg);
                if (pkgSection) {
                    // Try to extract label
                    const labelMatch = pkgSection.match(/applicationInfo.*label=([^\s,}]+)/);
                    if (labelMatch && !labelMatch[1].startsWith('0x')) {
                        labelMap[pkg] = labelMatch[1];
                    }
                }

                // Get icon path from APK if we have the APK path
                if (apkPaths[pkg]) {
                    // Icon is inside the APK - we'll need to extract it
                    // For now, just store the APK path - we'll extract on-demand
                    iconMap[pkg] = apkPaths[pkg];
                }
            });
        } catch (e) {
            console.warn("dumpsys package failed, using fallbacks");
        }

        return { labelMap, iconMap };
    } catch (e) {
        console.error("Failed to get app info:", e);
        return { labelMap: {}, iconMap: {} };
    }
};

/**
 * Extract a specific package's section from dumpsys package output
 */
const extractPackageSection = (dumpsysOutput, packageName) => {
    const lines = dumpsysOutput.split('\n');
    let inPackage = false;
    let section = '';

    for (const line of lines) {
        if (line.includes(`Package [${packageName}]`)) {
            inPackage = true;
        } else if (inPackage && line.startsWith('  Package [')) {
            // Start of next package
            break;
        }

        if (inPackage) {
            section += line + '\n';
        }
    }

    return section;
};

/**
 * Extract label from pm dump output (faster than aapt)
 */
const extractLabelFromDump = async (serial, packageName, dumpOutput) => {
    try {
        // pm dump gives us the application label in various places
        // Look for common patterns

        // Pattern 1: versionName or labelRes in the output
        const labelMatch = dumpOutput.match(/ApplicationInfo.*label=([^\s,}]+)/);
        if (labelMatch && labelMatch[1] && !labelMatch[1].startsWith('0x')) {
            return labelMatch[1];
        }

        // If we only have resource ID, we'd need to decode it
        // For now, return null to use fallback
        return null;
    } catch (e) {
        return null;
    }
};

/**
 * FASTEST METHOD: Get app label directly using cmd package (Android 7+)
 * Much faster than aapt or dumpsys
 */
export const getAppLabel = async (serial, packageName) => {
    try {
        // Try the fastest method first: direct package manager query
        const output = await runAdbCommand([
            "-s", serial, "shell",
            `cmd package resolve-activity -a android.intent.action.MAIN -c android.intent.category.LAUNCHER ${packageName} | grep label`
        ]);

        const match = output.match(/label='([^']+)'/);
        if (match) return match[1];
    } catch (e) {
        // Method failed, fall back
    }

    // Fallback: use package name's last segment
    return null;
};

// Remove the old slow getAppLabelDirectly and getAppLabelFromResource functions
// They are replaced by the faster methods above

/**
 * Get app icon as base64 encoded PNG
 */
export const getAppIcon = async (serial, packageName) => {
    try {
        // Get APK path
        const pathOut = await runAdbCommand([
            "-s", serial, "shell", "pm", "path", packageName
        ]);
        const pathMatch = pathOut.match(/package:(.*\.apk)/);

        if (!pathMatch) return null;

        const apkPath = pathMatch[1];

        // Extract icon using aapt
        const iconOut = await runAdbCommand([
            "-s", serial, "shell", "aapt", "dump", "badging", apkPath
        ]);

        const iconMatch = iconOut.match(/application-icon-\d+:'([^']+)'/);
        if (!iconMatch) return null;

        const iconPath = iconMatch[1];

        // Pull icon to temp location and convert to base64
        // This is complex - for now return null
        // TODO: Implement icon extraction
        return null;
    } catch (e) {
        console.error(`Failed to get icon for ${packageName}:`, e);
        return null;
    }
};

export const uninstallPackage = async (serial, pkgName) => {
    return runAdbCommand(["-s", serial, "uninstall", pkgName]);
};

export const installPackage = async (serial, filePath) => {
    return runAdbCommand(["-s", serial, "install", "-r", filePath]);
};

export const clearPackageData = async (serial, pkgName) => {
    return runAdbCommand(["-s", serial, "shell", "pm", "clear", pkgName]);
};

/**
 * Starts tracking ADB device connections/disconnections.
 * @param {function} onChange - Callback triggered when device list changes
 * @returns {Promise<function>} - Cleanup function to stop tracking
 */
export const startDeviceTracking = async (onChange) => {
    let child = null;

    try {
        const command = Command.create("adb", ["track-devices"]);

        command.stdout.on("data", () => {
            onChange();
        });

        child = await command.spawn();
    } catch (e) {
        console.error("Failed to start device tracking:", e);
    }

    return async () => {
        if (child) {
            try {
                await child.kill();
            } catch (e) {
                console.error("Failed to kill tracking process:", e);
            }
        }
    };
};

/**
 * Lists files in a specific directory on the device
 * @param {string} serial 
 * @param {string} path 
 * @returns {Promise<Array>} List of files
 */
export const listFiles = async (serial, path) => {
    try {
        const stdout = await runAdbCommand(["-s", serial, "shell", "ls", "-pl", path]);
        const lines = stdout.split("\n");

        const files = lines
            .filter(line => line.trim() !== "" && !line.startsWith("total"))
            .map(line => {
                const parts = line.trim().split(/\s+/);
                if (parts.length < 6) return null; // Not enough parts 
                if (!/^[d-]/.test(parts[0])) return null;

                let nameIndex = 7;
                // Heuristic: Check if parts[6] or parts[7] looks like time/year
                const timeRegex = /^(\d{2}:\d{2}|\d{4})$/;
                if (timeRegex.test(parts[6])) nameIndex = 7;
                else if (timeRegex.test(parts[7])) nameIndex = 8;
                else nameIndex = 7;

                if (parts.length < nameIndex + 1) return null;

                const name = parts.slice(nameIndex).join(" ");
                if (!name || name === "." || name === "..") return null;

                const isDir = parts[0].startsWith("d");
                const cleanName = name.endsWith('/') ? name.slice(0, -1) : name;

                // Fix path double slashes
                const fullPath = path.endsWith('/') ? path + cleanName : path + '/' + cleanName;

                return {
                    permissions: parts[0],
                    owner: parts[2],
                    group: parts[3],
                    size: parts[4],
                    date: parts[5] + " " + parts[6],
                    name: cleanName,
                    isDirectory: isDir,
                    path: fullPath
                };
            })
            .filter(f => f !== null);

        return files;
    } catch (e) {
        console.error("List files failed", e);
        return [];
    }
};

export const getMediaFiles = async (serial) => {
    try {
        const stdout = await runAdbCommand([
            "-s", serial, "shell", "content", "query",
            "--uri", "content://media/external/images/media",
            "--projection", "_id:_data:bucket_display_name:date_added:mime_type",
            "--sort", "date_added DESC"
        ]);

        return stdout.split("\n")
            .filter(l => l.startsWith("Row:"))
            .map(line => {
                const map = {};
                // Row: 0 _id=123, _data=/path/..., key=val...
                const content = line.substring(line.indexOf(" ") + 1); // remove "Row: "
                const pairs = content.split(", ");

                pairs.forEach(p => {
                    const eqIdx = p.indexOf("=");
                    if (eqIdx > -1) {
                        const key = p.substring(0, eqIdx).trim();
                        const val = p.substring(eqIdx + 1);
                        map[key] = val;
                    }
                });

                if (!map._data) return null;

                return {
                    id: map._id,
                    path: map._data,
                    album: map.bucket_display_name || "Unknown",
                    date: parseInt(map.date_added) * 1000,
                    mime: map.mime_type,
                    name: map._data.split("/").pop()
                };
            })
            .filter(i => i !== null);
    } catch (e) {
        console.error("Get media files failed", e);
        return [];
    }
};

export const pullFile = async (serial, devicePath, localPath) => {
    return runAdbCommand(["-s", serial, "pull", devicePath, localPath]);
};

export const pushFile = async (serial, localPath, devicePath) => {
    return runAdbCommand(["-s", serial, "push", localPath, devicePath]);
};

export const deleteFile = async (serial, devicePath) => {
    return runAdbCommand(["-s", serial, "shell", "rm", "-rf", devicePath]);
};

export const createDirectory = async (serial, path) => {
    return runAdbCommand(["-s", serial, "shell", "mkdir", "-p", path]);
};

export const renameFile = async (serial, oldPath, newPath) => {
    return runAdbCommand(["-s", serial, "shell", "mv", oldPath, newPath]);
};
