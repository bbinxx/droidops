import React, { useState, useEffect, useRef } from "react";
import { Smartphone } from "lucide-react";
import { getAppIcon } from "../lib/cache";
import { runAdbCommand } from "../lib/adb";

/**
 * Lazy-loaded app icon component
 * Extracts and caches app icons from APKs
 */
export const AppIcon = ({ serial, app }) => {
    const [iconSrc, setIconSrc] = useState(null);
    const [loading, setLoading] = useState(true);
    const mounted = useRef(true);

    useEffect(() => {
        mounted.current = true;
        loadIcon();

        return () => { mounted.current = false; };
    }, [serial, app.packageName]);

    const loadIcon = async () => {
        if (!app.iconPath) {
            setLoading(false);
            return;
        }

        try {
            // Extract icon from APK and cache it
            const iconPath = await extractIconFromAPK(serial, app.packageName, app.iconPath);

            if (iconPath && mounted.current) {
                const src = await getAppIcon(serial, app.packageName, iconPath);
                if (mounted.current) {
                    setIconSrc(src);
                }
            }
        } catch (e) {
            console.error(`Failed to load icon for ${app.packageName}:`, e);
        } finally {
            if (mounted.current) {
                setLoading(false);
            }
        }
    };

    if (iconSrc) {
        return (
            <div className="w-12 h-12 flex-shrink-0 rounded-xl overflow-hidden shadow-lg border border-gray-700">
                <img src={iconSrc} alt={app.appName} className="w-full h-full object-cover" />
            </div>
        );
    }

    // Fallback gradient icon
    return (
        <div className="w-12 h-12 flex-shrink-0 bg-gradient-to-br from-blue-500 to-purple-600 rounded-xl flex items-center justify-center shadow-lg">
            <Smartphone className="text-white" size={24} />
        </div>
    );
};

/**
 * Extract icon file path from APK
 * Uses aapt or alternative methods to find the icon resource
 */
const extractIconFromAPK = async (serial, packageName, apkPath) => {
    try {
        // Try using aapt dump badging to get icon path
        const badgingOut = await runAdbCommand([
            "-s", serial, "shell",
            `aapt dump badging ${apkPath} | grep application-icon`
        ]);

        // Look for the highest density icon
        const iconMatches = badgingOut.matchAll(/application-icon-(\d+):'([^']+)'/g);
        let bestIcon = null;
        let bestDensity = 0;

        for (const match of iconMatches) {
            const density = parseInt(match[1]);
            const iconPath = match[2];

            if (density > bestDensity) {
                bestDensity = density;
                bestIcon = iconPath;
            }
        }

        if (bestIcon) {
            // Icon path is relative to APK, extract it
            // We need to unzip the APK to get the icon file
            // For now, return the icon resource path
            return `${apkPath}:${bestIcon}`; // Special format for APK resource
        }

    } catch (e) {
        // aapt might not be available
        console.warn(`aapt not available for icon extraction: ${e.message}`);
    }

    return null;
};
