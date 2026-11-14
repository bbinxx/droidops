import React, { useState, useEffect } from "react";
import { getConnectedDevices, getFastbootDevices, startDeviceTracking, runScrcpy, checkScrcpyInstalled, installDependencies } from "../lib/adb";
import { RefreshCw, Smartphone, Cast } from "lucide-react";
import { STRINGS } from "../data/strings";
import { InstallModal } from "./InstallModal";

export function DeviceSelector({ selectedDevice, onSelect, className }) {
    const [devices, setDevices] = useState([]);
    const [loading, setLoading] = useState(false);

    // Dependency installation states
    const [installModalOpen, setInstallModalOpen] = useState(false);
    const [installProgress, setInstallProgress] = useState(0);
    const [installError, setInstallError] = useState(null);
    const [missingDeps, setMissingDeps] = useState([]);

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

    const triggerMirror = async (serial) => {
        console.log("[mirror] Starting scrcpy for device:", serial);
        try {
            await runScrcpy(serial, {
                bitRate: "8M",
                alwaysOnTop: true,
                stayAwake: true
            });
            console.log("[mirror] scrcpy launched OK");
        } catch (error) {
            console.error("[mirror] scrcpy error:", error);
            const errorMessage = error?.message || error?.toString() || "Unknown error";
            alert(`Failed to launch screen mirror:\n\n${errorMessage}\n\nCheck the developer console (F12) for full logs.`);
        }
    };

    const handleMirrorScreen = async () => {
        if (!selectedDevice || selectedDevice.type !== 'adb') {
            alert("Please select an ADB device to mirror screen");
            return;
        }

        // 1. Verify if Scrcpy is installed on the host
        const isScrcpyAvailable = await checkScrcpyInstalled();
        if (!isScrcpyAvailable) {
            // Ask permission to install
            const proceed = window.confirm(
                "Screen Mirroring requires 'scrcpy' to be installed on your Linux system.\n\nWould you like DroidOps to securely install Scrcpy via package manager?"
            );

            if (proceed) {
                setMissingDeps(["scrcpy"]);
                setInstallProgress(0);
                setInstallError(null);
                setInstallModalOpen(true);

                try {
                    await installDependencies({ installScrcpy: true }, (progress) => {
                        setInstallProgress(progress);
                    });
                    
                    // Auto-launch mirroring upon successful install!
                    setTimeout(() => {
                        setInstallModalOpen(false);
                        triggerMirror(selectedDevice.serial);
                    }, 1500);
                } catch (err) {
                    setInstallError(err?.message || err?.toString() || "Installation was aborted or failed");
                }
            }
            return;
        }

        // 2. Already installed, execute directly
        await triggerMirror(selectedDevice.serial);
    };

    return (
        <>
            <div className={`flex items-center gap-3 bg-[#16161a] border border-[#27272a] rounded-xl p-2 px-4 shadow-xl select-none ${className}`}>
                <div className="flex items-center gap-2 text-gray-400">
                    <Smartphone size={18} className="text-[#E95420] shrink-0" />
                    <span className="text-xs font-semibold uppercase tracking-wider text-gray-400">{STRINGS.device.label}</span>
                </div>

                <div className="relative">
                    <select
                        value={selectedDevice?.serial || ""}
                        onChange={(e) => {
                            const dev = devices.find(d => d.serial === e.target.value);
                            onSelect(dev || null);
                        }}
                        className="bg-[#0f0f12] border border-zinc-800 text-gray-200 text-sm rounded-lg cursor-pointer min-w-[220px] py-1.5 pl-3 pr-10 appearance-none focus:ring-2 focus:ring-[#E95420] focus:border-[#E95420] outline-none transition-all hover:border-zinc-700"
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
                    <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center px-3 text-gray-500">
                        <svg className="fill-current h-4 w-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">
                            <path d="M9.293 12.95l.707.707L15.657 8l-1.414-1.414L10 10.828 5.757 6.586 4.343 8z" />
                        </svg>
                    </div>
                </div>

                {/* Mirror Screen Button */}
                {selectedDevice && selectedDevice.type === 'adb' && (
                    <button
                        onClick={handleMirrorScreen}
                        className="flex items-center gap-2 px-3 py-1.5 bg-[#E95420] hover:bg-[#EB6536] text-white rounded-lg text-xs font-semibold transition-all shadow-md cursor-pointer hover:-translate-y-0.5 active:translate-y-0"
                        title="Mirror screen with Scrcpy"
                    >
                        <Cast size={14} />
                        <span>Mirror</span>
                    </button>
                )}

                <button
                    onClick={refresh}
                    className="p-1.5 hover:bg-zinc-800 rounded-lg text-gray-400 hover:text-[#E95420] transition-colors cursor-pointer"
                    title={STRINGS.device.refreshTooltip}
                >
                    <RefreshCw size={16} className={loading ? "animate-spin text-[#E95420]" : ""} />
                </button>
            </div>

            <InstallModal
                isOpen={installModalOpen}
                onClose={() => setInstallModalOpen(false)}
                progress={installProgress}
                error={installError}
                missingDeps={missingDeps}
            />
        </>
    );
}
