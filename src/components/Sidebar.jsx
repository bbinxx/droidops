import React from "react";
import { Terminal, Zap, Package, Smartphone, Home, Upload, FolderOpen, Images } from "lucide-react";
import { STRINGS } from "../data/strings";
import { ROUTES, APP_CONFIG } from "../data/config";

export function Sidebar({ activeTab, setActiveTab }) {
    const navItems = [
        { id: ROUTES.DASHBOARD, icon: <Home size={20} />, label: STRINGS.navigation.dashboard },
        { id: ROUTES.APPS, icon: <Package size={20} />, label: STRINGS.navigation.apps },
        { id: ROUTES.FILES, icon: <FolderOpen size={20} />, label: STRINGS.navigation.files },
        { id: ROUTES.GALLERY, icon: <Images size={20} />, label: STRINGS.navigation.gallery },
        { id: ROUTES.TERMINAL, icon: <Terminal size={20} />, label: STRINGS.navigation.terminal },
        { id: ROUTES.FASTBOOT, icon: <Zap size={20} />, label: STRINGS.navigation.fastboot },
        { id: ROUTES.SIDELOAD, icon: <Upload size={20} />, label: STRINGS.navigation.sideload },
    ];

    return (
        <aside className="w-64 bg-[#1a1a1f] border-r border-[#27272a] flex flex-col p-4 h-full">
            <div className="flex items-center gap-2 mb-8 px-2">
                <Smartphone className="w-6 h-6 text-blue-500" />
                <h1 className="text-xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-400 to-purple-600">
                    {APP_CONFIG.app.displayName}
                </h1>
            </div>

            <nav className="flex-1 flex flex-col gap-2">
                {navItems.map((item) => (
                    <button
                        key={item.id}
                        onClick={() => setActiveTab(item.id)}
                        className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg text-left transition-all duration-200 ${activeTab === item.id
                            ? "bg-[#27272a] text-white shadow-lg shadow-black/20"
                            : "text-gray-400 hover:bg-[#27272a]/50 hover:text-gray-200"
                            }`}
                    >
                        {item.icon}
                        <span className="font-medium">{item.label}</span>
                    </button>
                ))}
            </nav>

            {/* <div className="mt-auto pt-4 border-t border-[#27272a]">
                <div className="text-xs text-gray-500 text-center">v2.1.0</div>
            </div> */}
        </aside>
    );
}
