import React, { useEffect, useState } from "react";
import { getDeviceInfo, checkAdbStatus, installDependencies } from "../lib/adb";
import { Battery, Smartphone, Activity, CheckCircle2, AlertCircle, XCircle, Terminal, Copy, Check, ShieldAlert } from "lucide-react";
import { InstallModal } from "../components/InstallModal";

export function Dashboard({ selectedDevice }) {
    const [info, setInfo] = useState(null);
    const [loading, setLoading] = useState(false);
    const [adbStatus, setAdbStatus] = useState(null);
    const [checkingAdb, setCheckingAdb] = useState(false);
    const [copiedIndex, setCopiedIndex] = useState(null);

    // Dependency installation states
    const [installModalOpen, setInstallModalOpen] = useState(false);
    const [installProgress, setInstallProgress] = useState(0);
    const [installError, setInstallError] = useState(null);
    const [missingDeps, setMissingDeps] = useState([]);

    const runAdbDiagnostic = async () => {
        setCheckingAdb(true);
        const status = await checkAdbStatus();
        setAdbStatus(status);
        setCheckingAdb(false);
    };

    const handleInstallAdb = async () => {
        const proceed = window.confirm(
            "ADB and Fastboot are required to interact with Android devices, but they are not installed on your system.\n\nWould you like DroidOps to securely install them via package manager?"
        );

        if (proceed) {
            setMissingDeps(["ADB", "Fastboot"]);
            setInstallProgress(0);
            setInstallError(null);
            setInstallModalOpen(true);

            try {
                await installDependencies({ installAdb: true }, (progress) => {
                    setInstallProgress(progress);
                });
                
                // Re-run diagnostics automatically after install!
                setTimeout(() => {
                    setInstallModalOpen(false);
                    runAdbDiagnostic();
                }, 1500);
            } catch (err) {
                setInstallError(err?.message || err?.toString() || "Installation failed or was aborted");
            }
        }
    };

    useEffect(() => {
        if (selectedDevice?.serial && selectedDevice?.type === 'adb') {
            setLoading(true);
            getDeviceInfo(selectedDevice.serial).then((data) => {
                setInfo(data);
                setLoading(false);
            });
        } else {
            setInfo(null);
            runAdbDiagnostic();
        }
    }, [selectedDevice]);

    const handleCopy = (text, index) => {
        navigator.clipboard.writeText(text);
        setCopiedIndex(index);
        setTimeout(() => setCopiedIndex(null), 2000);
    };

    if (!selectedDevice) {
        const linuxSetupSteps = [
            {
                title: "1. Install ADB & Fastboot",
                desc: "Install the Android SDK platform tools from the package manager.",
                cmd: "sudo apt update && sudo apt install -y android-tools-adb android-tools-fastboot"
            },
            {
                title: "2. Configure USB Permissions (udev)",
                desc: "Download and configure udev rules to allow non-root USB access to your Android devices.",
                cmd: "sudo curl -SLo /etc/udev/rules.d/51-android.rules https://raw.githubusercontent.com/M0Rf30/android-udev-rules/master/51-android.rules && sudo chmod a+r /etc/udev/rules.d/51-android.rules"
            },
            {
                title: "3. Restart udev & Add Group Permissions",
                desc: "Reload the udev daemon rules and add your user account to the plugdev group.",
                cmd: "sudo udevadm control --reload-rules && sudo service udev restart && sudo usermod -aG plugdev $USER"
            }
        ];

        return (
            <div className="p-8 max-w-5xl mx-auto animate-fade-in space-y-8">
                <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 bg-[#1a1a1f] p-6 rounded-2xl border border-[#27272a] shadow-xl">
                    <div>
                        <h2 className="text-2xl font-bold text-white mb-2">ADB Connection Diagnostics</h2>
                        <p className="text-gray-400 text-sm">Please connect a device and select it from the top-right menu to get started.</p>
                    </div>
                    <button
                        onClick={runAdbDiagnostic}
                        disabled={checkingAdb}
                        className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-white rounded-lg text-sm font-medium border border-zinc-700 transition-colors cursor-pointer disabled:opacity-50"
                    >
                        {checkingAdb ? "Checking..." : "Re-Run Diagnostics"}
                    </button>
                </div>

                {/* Diagnostics Check Grid */}
                {adbStatus && (
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                        {/* Tauri Environment */}
                        <div className={`p-6 rounded-2xl border bg-[#1a1a1f] transition-all flex flex-col justify-between ${adbStatus.isTauri ? 'border-emerald-500/20' : 'border-amber-500/20'}`}>
                            <div className="space-y-4">
                                <div className="flex justify-between items-start">
                                    <span className="text-gray-400 text-xs font-semibold uppercase tracking-wider">Environment</span>
                                    {adbStatus.isTauri ? (
                                        <CheckCircle2 className="text-emerald-500" size={24} />
                                    ) : (
                                        <AlertCircle className="text-amber-500" size={24} />
                                    )}
                                </div>
                                <h3 className="text-lg font-bold text-white">
                                    {adbStatus.isTauri ? "Tauri Desktop App" : "Web Browser Mode"}
                                </h3>
                                <p className="text-gray-400 text-sm">
                                    {adbStatus.isTauri 
                                        ? "Running as native desktop shell. Shell integration and file transfer features are enabled."
                                        : "Tauri core libraries not detected. Run using 'npm run tauri dev' to enable hardware access."}
                                </p>
                            </div>
                        </div>

                        {/* ADB Status */}
                        <div className={`p-6 rounded-2xl border bg-[#1a1a1f] transition-all flex flex-col justify-between ${adbStatus.isInstalled ? 'border-emerald-500/20' : 'border-rose-500/20'}`}>
                            <div className="space-y-4">
                                <div className="flex justify-between items-start">
                                    <span className="text-gray-400 text-xs font-semibold uppercase tracking-wider">ADB Server</span>
                                    {adbStatus.isInstalled ? (
                                        <CheckCircle2 className="text-emerald-500" size={24} />
                                    ) : (
                                        <XCircle className="text-rose-500" size={24} />
                                    )}
                                </div>
                                <h3 className="text-lg font-bold text-white">
                                    {adbStatus.isInstalled ? "ADB Found & Ready" : "ADB Service Missing"}
                                </h3>
                                <p className="text-gray-400 text-sm">
                                    {adbStatus.isInstalled 
                                        ? "The Android Debug Bridge is installed and accessible in your system's PATH."
                                        : adbStatus.isTauri 
                                            ? "Unable to execute 'adb'. Please ensure 'adb' is installed and added to your system's environment PATH."
                                            : "ADB requires native system permissions. Launch as a desktop application to search for ADB."}
                                </p>
                            </div>
                            {!adbStatus.isInstalled && adbStatus.isTauri && (
                                <button
                                    onClick={handleInstallAdb}
                                    className="mt-4 w-full py-2 bg-gradient-to-r from-rose-600 to-red-500 hover:from-rose-500 hover:to-red-400 text-white rounded-xl text-xs font-bold transition-all shadow-md hover:-translate-y-0.5 active:translate-y-0 cursor-pointer border border-rose-500/20"
                                >
                                    Auto-Install ADB & Fastboot
                                </button>
                            )}
                        </div>

                        {/* Connected Devices */}
                        <div className={`p-6 rounded-2xl border bg-[#1a1a1f] transition-all flex flex-col justify-between ${adbStatus.devicesCount > 0 ? 'border-emerald-500/20' : 'border-amber-500/20'}`}>
                            <div className="space-y-4">
                                <div className="flex justify-between items-start">
                                    <span className="text-gray-400 text-xs font-semibold uppercase tracking-wider">Hardware</span>
                                    {adbStatus.devicesCount > 0 ? (
                                        <CheckCircle2 className="text-emerald-500" size={24} />
                                    ) : (
                                        <AlertCircle className="text-amber-500" size={24} />
                                    )}
                                </div>
                                <h3 className="text-lg font-bold text-white">
                                    {adbStatus.devicesCount > 0 ? `${adbStatus.devicesCount} Device(s) Connected` : "No Devices Detected"}
                                </h3>
                                <p className="text-gray-400 text-sm">
                                    {adbStatus.devicesCount > 0 
                                        ? "Hardware handshakes successful. Device serials and endpoints detected."
                                        : "No USB or wireless debugging connections found. Connect your device with USB debugging enabled."}
                                </p>
                            </div>
                        </div>
                    </div>
                )}

                {/* Linux ADB Setup Guide */}
                <div className="bg-[#1a1a1f] rounded-2xl border border-[#27272a] shadow-xl overflow-hidden animate-slide-up">
                    <div className="p-6 border-b border-[#27272a] flex items-center gap-3 bg-gradient-to-r from-orange-500/10 to-transparent">
                        <ShieldAlert className="text-orange-500" size={24} />
                        <div>
                            <h3 className="text-lg font-bold text-white">Linux ADB & USB Permissions Setup</h3>
                            <p className="text-gray-400 text-xs mt-0.5">Follow these commands to configure ADB, fastboot, and udev rules on your Linux host system.</p>
                        </div>
                    </div>

                    <div className="p-6 space-y-6">
                        {linuxSetupSteps.map((step, idx) => (
                            <div key={idx} className="space-y-3">
                                <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-1">
                                    <h4 className="text-sm font-semibold text-gray-200">{step.title}</h4>
                                    <p className="text-xs text-gray-400">{step.desc}</p>
                                </div>
                                <div className="relative flex items-center bg-[#0c0c0e] rounded-xl border border-zinc-800/80 pl-4 pr-12 py-3.5 select-all">
                                    <Terminal size={14} className="text-zinc-500 mr-2.5 shrink-0" />
                                    <code className="text-xs font-mono text-zinc-300 overflow-x-auto whitespace-pre select-text flex-1 pr-4 py-0.5 scrollbar-thin">
                                        {step.cmd}
                                    </code>
                                    <button
                                        onClick={() => handleCopy(step.cmd, idx)}
                                        className="absolute right-3 p-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 hover:text-white rounded-lg border border-zinc-700 transition-all cursor-pointer"
                                        title="Copy Command"
                                    >
                                        {copiedIndex === idx ? <Check size={14} className="text-emerald-500" /> : <Copy size={14} />}
                                    </button>
                                </div>
                            </div>
                        ))}

                        <div className="mt-4 p-4 bg-orange-950/20 border border-orange-500/20 rounded-xl text-xs text-orange-400 leading-relaxed flex gap-3">
                            <AlertCircle size={16} className="shrink-0 mt-0.5" />
                            <div>
                                <span className="font-semibold block mb-0.5">Post-Configuration Note:</span>
                                After running these commands and adding your user to the <code className="bg-orange-950/40 px-1 rounded font-mono font-bold text-white">plugdev</code> group, you must **restart your system** or **log out and log back in** for the group permissions to take effect.
                            </div>
                        </div>
                    </div>
                </div>
                <InstallModal
                    isOpen={installModalOpen}
                    onClose={() => setInstallModalOpen(false)}
                    progress={installProgress}
                    error={installError}
                    missingDeps={missingDeps}
                />
            </div>
        );
    }

    if (selectedDevice.type === 'fastboot') {
        return (
            <div className="p-8">
                <h2 className="text-2xl font-bold mb-6 text-white">Device Dashboard</h2>
                <div className="bg-orange-900/20 border border-orange-500/30 p-6 rounded-xl">
                    <h3 className="text-xl font-bold text-orange-400 mb-2">Fastboot Mode</h3>
                    <p className="text-gray-300">Device is in Fastboot/Bootloader mode. Limited information available.</p>
                    <p className="mt-2 text-sm text-gray-400">Serial: {selectedDevice.serial}</p>
                </div>
            </div>
        );
    }

    return (
        <div className="p-8 animate-fade-in">
            <h2 className="text-2xl font-bold mb-6 text-white">Device Overview</h2>

            {loading ? (
                <div className="text-center py-10 text-gray-400">Loading device info...</div>
            ) : info ? (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    <InfoCard
                        icon={<Smartphone className="text-blue-400" />}
                        label="Model"
                        value={info.model}
                    />
                    <InfoCard
                        icon={<Activity className="text-green-400" />}
                        label="Android Version"
                        value={info.androidVersion}
                    />
                    <InfoCard
                        icon={<Battery className={getBatteryColor(info.batteryLevel)} />}
                        label="Battery"
                        value={`${info.batteryLevel}% (${info.batteryStatus})`}
                    />
                </div>
            ) : (
                <div className="text-red-400">Failed to load device info</div>
            )}

            <div className="mt-8 bg-[#1a1a1f] p-6 rounded-xl border border-[#27272a]">
                <h3 className="text-lg font-semibold mb-4 text-gray-200">Quick Actions</h3>
                <p className="text-gray-500 text-sm">Select a tab from the sidebar to manage apps, files, or execute commands.</p>
            </div>
        </div>
    );
}

function InfoCard({ icon, label, value }) {
    return (
        <div className="bg-[#1a1a1f] p-6 rounded-xl border border-[#27272a] shadow-lg flex items-center gap-4 hover:border-blue-500/50 transition-colors">
            <div className="p-3 bg-black/30 rounded-lg">{icon}</div>
            <div>
                <p className="text-sm text-gray-400">{label}</p>
                <p className="text-lg font-semibold text-white">{value}</p>
            </div>
        </div>
    );
}

function getBatteryColor(level) {
    const l = parseInt(level);
    if (isNaN(l)) return "text-gray-400";
    if (l > 60) return "text-green-400";
    if (l > 20) return "text-yellow-400";
    return "text-red-500";
}
