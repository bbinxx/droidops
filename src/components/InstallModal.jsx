import React from "react";
import { CheckCircle2, XCircle, Loader2, ShieldAlert, Cpu } from "lucide-react";

export function InstallModal({ isOpen, onClose, progress, status, error, missingDeps }) {
    if (!isOpen) return null;

    const getStatusText = () => {
        if (error) return "Installation Failed";
        if (progress === 100) return "Installation Complete!";
        if (progress > 85) return "Configuring packages...";
        if (progress > 50) return "Extracting and unpacking dependencies...";
        if (progress > 10) return "Downloading required packages...";
        if (progress > 0) return "Waiting for password authorization...";
        return "Preparing installer...";
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-md animate-fade-in px-4">
            <div className="bg-[#121216] border border-[#27272a] rounded-2xl w-full max-w-md p-6 shadow-2xl relative overflow-hidden animate-slide-up">
                {/* Visual Accent Glow */}
                <div className={`absolute top-0 inset-x-0 h-1.5 bg-gradient-to-r ${
                    error ? "from-red-500 to-rose-600" : 
                    progress === 100 ? "from-emerald-500 to-teal-400" : 
                    "from-[#E95420] to-[#EB6536]"
                }`} />

                <div className="space-y-6">
                    {/* Header */}
                    <div className="flex items-start gap-4">
                        <div className={`p-3 rounded-xl ${
                            error ? "bg-red-500/10 text-red-500" :
                            progress === 100 ? "bg-emerald-500/10 text-emerald-500" :
                            "bg-orange-500/10 text-[#E95420]"
                        }`}>
                            {error ? <XCircle size={24} /> :
                             progress === 100 ? <CheckCircle2 size={24} /> :
                             <Cpu className="animate-pulse" size={24} />}
                        </div>
                        <div>
                            <h3 className="text-lg font-bold text-white">
                                {progress === 100 ? "Dependencies Ready" : "Installing Dependencies"}
                            </h3>
                            <p className="text-gray-400 text-xs mt-0.5">
                                {missingDeps && missingDeps.length > 0 
                                    ? `Setting up ${missingDeps.join(" & ")} on your Linux host` 
                                    : "Configuring system requirements"}
                            </p>
                        </div>
                    </div>

                    {/* Progress Visualizer */}
                    <div className="space-y-2">
                        <div className="flex justify-between items-baseline">
                            <span className={`text-xs font-semibold uppercase tracking-wider ${
                                error ? "text-red-400" : 
                                progress === 100 ? "text-emerald-400" : 
                                "text-orange-400"
                            }`}>
                                {getStatusText()}
                            </span>
                            <span className="text-sm font-mono font-bold text-white">{progress}%</span>
                        </div>

                        {/* Progress Bar Track */}
                        <div className="h-2 bg-[#1b1b22] rounded-full overflow-hidden border border-zinc-800/80">
                            <div 
                                className={`h-full rounded-full transition-all duration-300 ${
                                    error ? "bg-gradient-to-r from-red-500 to-rose-600" :
                                    progress === 100 ? "bg-gradient-to-r from-emerald-500 to-teal-400" :
                                    "bg-gradient-to-r from-[#E95420] to-[#EB6536]"
                                }`}
                                style={{ width: `${progress}%` }}
                            />
                        </div>
                    </div>

                    {/* Details Box */}
                    {error ? (
                        <div className="p-4 bg-red-950/20 border border-red-500/20 rounded-xl text-xs text-red-400 flex gap-2.5">
                            <ShieldAlert size={16} className="shrink-0 mt-0.5" />
                            <div>
                                <span className="font-semibold block mb-0.5">Error Log:</span>
                                {error}
                            </div>
                        </div>
                    ) : progress < 100 ? (
                        <div className="p-4 bg-[#1b1b22] border border-zinc-800 rounded-xl flex items-center justify-between text-xs text-gray-400">
                            <div className="flex items-center gap-2.5">
                                <Loader2 className="animate-spin text-orange-400" size={14} />
                                <span>Do not close this application window.</span>
                            </div>
                        </div>
                    ) : (
                        <div className="p-4 bg-emerald-950/20 border border-emerald-500/20 rounded-xl text-xs text-emerald-400 leading-relaxed">
                            <span className="font-semibold block mb-0.5">Success!</span>
                            All selected packages have been successfully configured in your system. Mirroring and diagnostics functions are fully active.
                        </div>
                    )}

                    {/* Actions */}
                    <div className="flex justify-end pt-2">
                        {error ? (
                            <button
                                onClick={onClose}
                                className="px-4 py-2 bg-red-600 hover:bg-red-500 text-white rounded-lg text-xs font-semibold transition-colors"
                            >
                                Close & Dismiss
                            </button>
                        ) : progress === 100 ? (
                            <button
                                onClick={onClose}
                                className="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white rounded-lg text-xs font-semibold transition-colors"
                            >
                                Get Started
                            </button>
                        ) : null}
                    </div>
                </div>
            </div>
        </div>
    );
}
