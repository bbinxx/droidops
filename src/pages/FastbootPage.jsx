import React, { useState } from "react";
import { runAdbCommand, runFastbootCommand } from "../lib/adb";
import { Zap, RotateCcw, Monitor } from "lucide-react";

export function FastbootPage({ selectedDevice }) {

    const action = async (fn, name) => {
        if (!selectedDevice) return alert("No device");
        try {
            await fn();
            alert(`${name} Command Sent`);
        } catch (e) {
            alert("Error: " + e);
        }
    };

    const adbReboot = (mode) => runAdbCommand(["-s", selectedDevice.serial, "reboot", mode]);
    const fastbootReboot = () => runFastbootCommand(["-s", selectedDevice.serial, "reboot"]);

    if (selectedDevice?.type === 'fastboot') {
        return (
            <div className="p-8 animate-fade-in">
                <h2 className="text-2xl font-bold mb-6 text-white">Fastboot Actions</h2>
                <div className="grid grid-cols-2 gap-4">
                    <ActionCard
                        icon={<Zap />}
                        title="Reboot System"
                        desc="Reboot from Fastboot to System"
                        onClick={() => action(fastbootReboot, "Reboot System")}
                    />
                </div>
            </div>
        );
    }

    return (
        <div className="p-8 animate-fade-in">
            <h2 className="text-2xl font-bold mb-6 text-white">Power Menu</h2>
            <div className="grid grid-cols-2 gap-4">
                <ActionCard
                    icon={<Monitor />}
                    title="Reboot System"
                    desc="Normal Reboot"
                    onClick={() => action(() => runAdbCommand(["-s", selectedDevice.serial, "reboot"]), "Reboot")}
                />
                <ActionCard
                    icon={<Zap />}
                    title="Reboot Bootloader"
                    desc="Enter Fastboot Mode"
                    onClick={() => action(() => adbReboot("bootloader"), "Reboot Bootloader")}
                />
                <ActionCard
                    icon={<RotateCcw />}
                    title="Reboot Recovery"
                    desc="Enter Recovery Mode"
                    onClick={() => action(() => adbReboot("recovery"), "Reboot Recovery")}
                />
            </div>
        </div>
    );
}

const ActionCard = ({ icon, title, desc, onClick }) => (
    <div onClick={onClick} className="bg-[#1a1a1f] hover:bg-[#27272a] border border-[#27272a] p-5 rounded-lg cursor-pointer transition-all hover:-translate-y-1 group">
        <div className="text-blue-500 mb-3 group-hover:text-amber-500 transition-colors">{icon}</div>
        <h3 className="font-semibold text-lg mb-1 text-white">{title}</h3>
        <p className="text-sm text-gray-400">{desc}</p>
    </div>
);
