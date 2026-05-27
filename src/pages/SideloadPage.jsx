import React from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { runAdbCommand } from "../lib/adb";
import { Upload } from "lucide-react";

export function SideloadPage({ selectedDevice }) {
    const handleSideload = async () => {
        if (!selectedDevice) return alert("Select a device first");

        const file = await open({
            multiple: false,
            filters: [{ name: "ZIP", extensions: ["zip"] }],
        });

        if (file) {
            if (!window.confirm(`Sideload ${file}? This cannot be undone.`)) return;
            try {
                // file might be object or string depending on version, 
                // but shell plugin usually takes string path.
                await runAdbCommand(["-s", selectedDevice.serial, "sideload", file]);
                alert("Sideload Complete");
            } catch (e) {
                alert("Sideload Failed: " + e);
            }
        }
    };

    return (
        <div className="h-full flex flex-col p-8 animate-fade-in items-center justify-center">
            <div className="max-w-md text-center">
                <h2 className="text-2xl font-bold mb-4 text-white">ADB Sideload</h2>
                <p className="text-gray-400 mb-8">
                    Flash OTA updates or ZIP packages in Recovery mode.
                    Ensure your device is in 'Apply update from ADB' mode.
                </p>

                <button
                    onClick={handleSideload}
                    disabled={!selectedDevice}
                    className="bg-blue-600 hover:bg-blue-500 text-white px-8 py-4 rounded-xl flex items-center gap-3 mx-auto disabled:opacity-50 disabled:cursor-not-allowed transition-all hover:scale-105 shadow-lg shadow-blue-900/20"
                >
                    <Upload size={24} />
                    <span className="font-semibold text-lg">Select ZIP File</span>
                </button>
            </div>
        </div>
    );
}
