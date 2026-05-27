import React, { useState } from "react";
import { runAdbCommand } from "../lib/adb";

export function ShellPage({ selectedDevice }) {
    const [output, setOutput] = useState("");
    const [cmd, setCmd] = useState("");

    const handleRun = async (e) => {
        e.preventDefault();
        if (!selectedDevice) return;

        setOutput(prev => prev + `> adb -s ${selectedDevice.serial} shell ${cmd}\n`);
        try {
            const res = await runAdbCommand(["-s", selectedDevice.serial, "shell", ...cmd.split(" ")]);
            setOutput(prev => prev + res + "\n");
        } catch (err) {
            setOutput(prev => prev + "Err: " + err + "\n");
        }
        setCmd("");
    };

    return (
        <div className="flex flex-col h-full p-6 animate-fade-in">
            <h2 className="text-2xl font-bold mb-4 text-white">ADB Shell</h2>
            <div className="flex-1 bg-black/50 border border-[#27272a] rounded-lg p-4 font-mono text-sm text-green-400 overflow-auto whitespace-pre-wrap mb-4">
                {output || "Type a command below..."}
            </div>
            <form onSubmit={handleRun} className="flex gap-2">
                <input
                    value={cmd}
                    onChange={e => setCmd(e.target.value)}
                    placeholder="ls -la /sdcard/"
                    className="flex-1 bg-[#1a1a1f] border border-[#27272a] rounded p-3 text-white font-mono focus:border-blue-500 outline-none"
                />
                <button type="submit" disabled={!selectedDevice} className="bg-blue-600 px-6 rounded font-semibold text-white hover:bg-blue-500 disabled:opacity-50">
                    Run
                </button>
            </form>
        </div>
    );
}
