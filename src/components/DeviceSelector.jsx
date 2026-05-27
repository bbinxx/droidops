import React, { useState, useEffect } from "react";
import { getConnectedDevices, getFastbootDevices, startDeviceTracking, runScrcpy } from "../lib/adb";
import { RefreshCw, Smartphone, Cast } from "lucide-react";
import { STRINGS } from "../data/strings";

export function DeviceSelector({ selectedDevice, onSelect, className }) {
    const [devices, setDevices] = useState([]);
    const [loading, setLoading] = useState(false);

    const refresh = async () => {
        setLoading(true);
        const [adbDevs, fastbootDevs] = await Promise.all([
            getConnectedDevices(),
            getFastbootDevices()
        ]);
        const all = [...adbDevs, ...fastbootDevs];
        setDevices(all);

        // Auto Select first if none selected or selection lost
        if (all.length > 0 && (!selectedDevice || !all.find(d => d.serial === selectedDevice.serial))) {
            onSelect(all[0]);
        } else if (all.length === 0) {
            onSelect(null);
        }

        setLoading(false);
    };

    useEffect(() => {
        let cleanup = null;
        let debounceTimer = null;

        const handleChange = () => {
            if (debounceTimer) clearTimeout(debounceTimer);
            debounceTimer = setTimeout(() => {
                refresh();
            }, 500);
        };

        const initTracking = async () => {
            await refresh(); // Initial check
            cleanup = await startDeviceTracking(handleChange);
        };

        initTracking();

        return () => {
            if (cleanup) cleanup();
            if (debounceTimer) clearTimeout(debounceTimer);
        };
    }, []); // eslint-disable-line

    const handleMirrorScreen = async () => {
        if (!selectedDevice || selectedDevice.type !== 'adb') {
            alert("Please select an ADB device to mirror screen");
            return;
        }

        try {
            console.log("Starting scrcpy for device:", selectedDevice.serial);
            await runScrcpy(selectedDevice.serial, {
                maxSize: 1920,  // Max resolution
                bitRate: "8M",  // 8 Mbps
                alwaysOnTop: true,
                stayAwake: true
            });
            console.log("Scrcpy started successfully");
        } catch (error) {
            console.error("Scrcpy error:", error);
            const errorMessage = error?.message || error?.toString() || "Unknown error";
            alert(`Failed to launch Scrcpy:\n\n${errorMessage}`);
        }
    };

    return (
        <div className={`flex items-center gap-3 bg-white border border-gray-200 rounded-lg p-2 px-4 shadow-sm ${className}`}>
            <div className="flex items-center gap-2 text-gray-600">
                <Smartphone size={18} className="text-[#E95420]" />
                <span className="text-sm font-medium text-gray-700">{STRINGS.device.label}</span>
            </div>

            <div className="relative">
                <select
                    value={selectedDevice?.serial || ""}
                    onChange={(e) => {
                        const dev = devices.find(d => d.serial === e.target.value);
                        onSelect(dev || null);
                    }}
                    className="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-md cursor-pointer min-w-[250px] py-2 pl-3 pr-10 appearance-none focus:ring-2 focus:ring-[#E95420] focus:border-[#E95420] outline-none transition-all"
                >
                    {devices.length === 0 ? (
                        <option value="">{STRINGS.device.noDevicesFound}</option>
                    ) : (
                        devices.map(d => (
                            <option key={d.serial} value={d.serial}>
                                {d.model} ({d.serial}) - {d.type.toUpperCase()}
                            </option>
                        ))
                    )}
                </select>
                {/* Custom Arrow */}
                <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center px-3 text-gray-400">
                    <svg className="fill-current h-4 w-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">
                        <path d="M9.293 12.95l.707.707L15.657 8l-1.414-1.414L10 10.828 5.757 6.586 4.343 8z" />
                    </svg>
                </div>
            </div>

            {/* Mirror Screen Button */}
            {selectedDevice && selectedDevice.type === 'adb' && (
                <button
                    onClick={handleMirrorScreen}
                    className="flex items-center gap-2 px-3 py-2 bg-[#E95420] hover:bg-[#EB6536] text-white rounded-md text-sm font-medium transition-colors shadow-sm"
                    title="Mirror screen with Scrcpy"
                >
                    <Cast size={16} />
                    <span>Mirror</span>
                </button>
            )}

            <button
                onClick={refresh}
                className="p-2 hover:bg-gray-100 rounded-md text-gray-600 hover:text-[#E95420] transition-colors"
                title={STRINGS.device.refreshTooltip}
            >
                <RefreshCw size={18} className={loading ? "animate-spin" : ""} />
            </button>
        </div>
    );
}
