import React, { useEffect, useState } from "react";
import { getInstalledPackages, uninstallPackage, clearPackageData, installPackage } from "../lib/adb";
import { Search, Trash2, Eraser, RefreshCw, Package, Upload } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { STRINGS } from "../data/strings";
import { confirmUninstall, confirmClearData, alertDialog } from "../utils/dialog";
import { AppIcon } from "../components/AppIcon";

export function AppManager({ selectedDevice }) {
    const [apps, setApps] = useState([]);
    const [loading, setLoading] = useState(false);
    const [filter, setFilter] = useState("");
    const [type, setType] = useState("user"); // user, system, all

    useEffect(() => {
        loadApps();
    }, [selectedDevice, type]);

    const loadApps = async () => {
        if (!selectedDevice || selectedDevice.type !== 'adb') return;
        setLoading(true);
        const list = await getInstalledPackages(selectedDevice.serial, type);
        setApps(list);
        setLoading(false);
    };

    const handleInstall = async () => {
        const file = await open({
            multiple: false,
            filters: [{ name: "APK", extensions: ["apk"] }]
        });
        if (file) {
            try {
                setLoading(true);
                await installPackage(selectedDevice.serial, file);
                alertDialog(STRINGS.apps.installSuccess);
                loadApps();
            } catch (e) {
                alertDialog(STRINGS.apps.installFailed + ": " + e);
            } finally {
                setLoading(false);
            }
        }
    };

    const handleUninstall = async (app) => {
        if (!confirmUninstall(app.appName)) return;
        try {
            await uninstallPackage(selectedDevice.serial, app.packageName);
            alertDialog(STRINGS.apps.uninstallSuccess);
            loadApps();
        } catch (e) {
            alertDialog(STRINGS.apps.uninstallFailed + ": " + e);
        }
    };

    const handleClearData = async (app) => {
        if (!confirmClearData(app.appName)) return;
        try {
            await clearPackageData(selectedDevice.serial, app.packageName);
            alertDialog(STRINGS.apps.clearDataSuccess);
        } catch (e) {
            alertDialog(STRINGS.apps.clearDataFailed + ": " + e);
        }
    };

    const filteredApps = apps.filter(app =>
        app.appName.toLowerCase().includes(filter.toLowerCase()) ||
        app.packageName.toLowerCase().includes(filter.toLowerCase())
    );

    if (!selectedDevice || selectedDevice.type !== 'adb') {
        return (
            <div className="h-full flex flex-col items-center justify-center text-gray-500">
                <Package size={48} className="mb-4 opacity-50" />
                <p>{STRINGS.errors.noDeviceSelected}</p>
            </div>
        );
    }

    return (
        <div className="h-full flex flex-col p-6 animate-fade-in">
            <div className="flex justify-between items-center mb-6">
                <h2 className="text-2xl font-bold text-white">{STRINGS.apps.title}</h2>

                <div className="flex gap-4 items-center">
                    <div className="relative">
                        <Search className="absolute left-3 top-2.5 text-gray-400" size={16} />
                        <input
                            type="text"
                            placeholder={STRINGS.common.search}
                            className="bg-[#1a1a1f] border border-[#27272a] rounded-lg pl-9 pr-4 py-2 text-sm text-white focus:border-blue-500 outline-none w-64"
                            value={filter}
                            onChange={e => setFilter(e.target.value)}
                        />
                    </div>

                    <select
                        value={type}
                        onChange={(e) => setType(e.target.value)}
                        className="bg-[#1a1a1f] border border-[#27272a] text-white text-sm rounded-lg px-3 py-2 outline-none focus:border-blue-500"
                    >
                        <option value="user">{STRINGS.apps.userApps}</option>
                        <option value="system">{STRINGS.apps.systemApps}</option>
                        <option value="all">{STRINGS.apps.allApps}</option>
                    </select>

                    <button onClick={loadApps} className="p-2 hover:bg-[#27272a] rounded-lg text-gray-400 hover:text-white transition-colors" title={STRINGS.files.refresh}>
                        <RefreshCw size={18} className={loading ? "animate-spin" : ""} />
                    </button>

                    <button onClick={handleInstall} className="flex items-center gap-2 bg-blue-600 hover:bg-blue-500 text-white px-3 py-2 rounded-lg text-sm font-medium transition-colors">
                        <Upload size={16} /> {STRINGS.sideload.install}
                    </button>
                </div>
            </div>

            <div className="flex-1 overflow-auto bg-[#1a1a1f] rounded-xl border border-[#27272a]">
                {loading ? (
                    <div className="p-10 text-center text-gray-500">{STRINGS.common.loading}</div>
                ) : filteredApps.length === 0 ? (
                    <div className="p-10 text-center text-gray-500">No apps found</div>
                ) : (
                    <div className="divide-y divide-[#27272a]">
                        {filteredApps.map(app => (
                            <div key={app.packageName} className="p-4 flex items-center justify-between hover:bg-black/20 transition-colors group">
                                <div className="flex items-center gap-3 flex-1 min-w-0">
                                    {/* App Icon */}
                                    <AppIcon serial={selectedDevice.serial} app={app} />

                                    {/* App Info */}
                                    <div className="flex-1 min-w-0">
                                        <h3 className="text-white font-medium truncate">{app.appName}</h3>
                                        <p className="text-gray-500 text-xs font-mono truncate">{app.packageName}</p>
                                    </div>
                                </div>

                                {/* Actions */}
                                <div className="flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                    <button
                                        onClick={() => handleClearData(app)}
                                        className="p-2 text-yellow-500 hover:bg-yellow-500/10 rounded-lg transition-colors"
                                        title={STRINGS.apps.clearData}
                                    >
                                        <Eraser size={16} />
                                    </button>
                                    <button
                                        onClick={() => handleUninstall(app)}
                                        className="p-2 text-red-500 hover:bg-red-500/10 rounded-lg transition-colors"
                                        title={STRINGS.apps.uninstall}
                                    >
                                        <Trash2 size={16} />
                                    </button>
                                </div>
                            </div>
                        ))}
                    </div>
                )}
            </div>
            <div className="mt-2 text-xs text-gray-500 px-1">
                Showing {filteredApps.length} of {apps.length} packages.
            </div>
        </div>
    );
}
