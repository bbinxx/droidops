import { appCacheDir } from "@tauri-apps/api/path";
import { convertFileSrc } from "@tauri-apps/api/core";
import { mkdir, exists, readFile, writeFile } from "@tauri-apps/plugin-fs";
import { join } from "@tauri-apps/api/path";
import { pullFile } from "./adb";
import PQueue from "p-queue";

// Limit concurrent downloads to avoid choking ADB/UI
const queue = new PQueue({ concurrency: 3 });

/**
 * Gets a local source URL for a thumbnail of the device file.
 * Downloads and caches the file if necessary.
 * 
 * @param {object} device // { serial }
 * @param {string} filePath // e.g. /sdcard/DCIM/Camera/IMG_001.jpg
 * @param {string} fileId // Unique ID for cache naming (e.g. name + size/date hash, or just name)
 * @returns {Promise<string|null>} Asset URL or null on failure
 */
export const getThumbnail = async (serial, filePath, fileName) => {
    try {
        const cacheDir = await appCacheDir();
        // Ensure cache directory exists
        // Note: appCacheDir usually exists, but subfolder might not
        const thumbDir = await join(cacheDir, "thumbnails");
        const dirExists = await exists(thumbDir);
        if (!dirExists) {
            await mkdir(thumbDir, { recursive: true });
        }

        // Simple sanitation for filename
        const safeName = fileName.replace(/[^a-z0-9.]/gi, '_');
        const localPath = await join(thumbDir, safeName);

        const isCached = await exists(localPath);

        if (isCached) {
            return convertFileSrc(localPath);
        }

        // Not cached, queue download
        return queue.add(async () => {
            try {
                // Double check existence in case another task finished it
                if (await exists(localPath)) return convertFileSrc(localPath);

                console.log(`Caching thumbnail: ${fileName}`);

                // ADB Pull
                // NOTE: We are pulling the FULL file. 
                // Creating a true thumbnail would require an on-device script or shell resizing
                // which is complex. For local-network ADB or USB, pulling typical 2-5MB photos is passable
                // if concurrency is low.
                await pullFile(serial, filePath, localPath);

                return convertFileSrc(localPath);
            } catch (e) {
                console.error(`Failed to cache ${fileName}:`, e);
                return null;
            }
        });

    } catch (e) {
        console.error("Thumbnail error:", e);
        return null;
    }
};

/**
 * Gets app icon from device and caches it locally
 * @param {string} serial - Device serial
 * @param {string} packageName - Package name
 * @param {string} iconPath - Path to icon on device (from APK)
 * @returns{Promise<string|null>} - Local asset URL or null
 */
export const getAppIcon = async (serial, packageName, iconPath) => {
    try {
        const cacheDir = await appCacheDir();
        const iconDir = await join(cacheDir, "app-icons");

        const dirExists = await exists(iconDir);
        if (!dirExists) {
            await mkdir(iconDir, { recursive: true });
        }

        // Use package name + hash of icon path for cache key
        const safeName = packageName.replace(/\./g, '_') + '.png';
        const localPath = await join(iconDir, safeName);

        const isCached = await exists(localPath);
        if (isCached) {
            return convertFileSrc(localPath);
        }

        // Not cached, queue download
        return queue.add(async () => {
            try {
                // Double check
                if (await exists(localPath)) return convertFileSrc(localPath);

                console.log(`Caching icon for: ${packageName}`);

                // Pull icon from device
                await pullFile(serial, iconPath, localPath);

                return convertFileSrc(localPath);
            } catch (e) {
                console.error(`Failed to cache icon for ${packageName}:`, e);
                return null;
            }
        });

    } catch (e) {
        console.error("App icon error:", e);
        return null;
    }
};
