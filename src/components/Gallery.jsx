import React, { useState, useEffect } from "react";
import { getMediaFiles, pullFile } from "../lib/adb";
import { Image, Grid, Clock, Folder, Download, X, Maximize2 } from "lucide-react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { tempDir, join } from "@tauri-apps/api/path";
import { save } from '@tauri-apps/plugin-dialog';

export function Gallery({ selectedDevice }) {
    const [tab, setTab] = useState("recent"); // recent | albums
    const [media, setMedia] = useState([]);
    const [albums, setAlbums] = useState({});
    const [loading, setLoading] = useState(false);
    const [preview, setPreview] = useState(null); // { file, src }
    const [previewLoading, setPreviewLoading] = useState(false);

    useEffect(() => {
        if (!selectedDevice) return;
        fetchMedia();
    }, [selectedDevice]);

    const fetchMedia = async () => {
        setLoading(true);
        const files = await getMediaFiles(selectedDevice.serial);
        setMedia(files);

        // Group by album
        const alb = {};
        files.forEach(f => {
            if (!alb[f.album]) alb[f.album] = [];
            alb[f.album].push(f);
        });
        setAlbums(alb);
        setLoading(false);
    };

    const handlePreview = async (file) => {
        try {
            setPreviewLoading(true);
            setPreview({ file, src: null }); // Open modal with loader

            // Pull to temp
            const temp = await tempDir();
            const fileName = `preview_${file.name}`;
            const localPath = await join(temp, fileName);

            await pullFile(selectedDevice.serial, file.path, localPath);
            const assetUrl = convertFileSrc(localPath);

            setPreview({ file, src: assetUrl, localPath }); // localPath for saving if needed
        } catch (e) {
            console.error("Preview failed", e);
            alert("Failed to load image preview");
            setPreview(null);
        } finally {
            setPreviewLoading(false);
        }
    };

    const handleDownload = async (file) => {
        try {
            const targetPath = await save({
                defaultPath: file.name,
                filters: [{ name: 'Image', extensions: ['jpg', 'png', 'jpeg', 'webp'] }]
            });

            if (targetPath) {
                await pullFile(selectedDevice.serial, file.path, targetPath);
                alert("Download Complete");
            }
        } catch (e) {
            console.error(e);
            alert("Download failed");
        }
    };

    if (!selectedDevice) return <div className="p-10 text-center text-gray-500">Connect a device to view gallery</div>;

    const renderRecent = () => (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(100px,1fr))] gap-2">
            {media.map(file => (
                <div
                    key={file.id}
                    className="aspect-square bg-[#27272a] rounded cursor-pointer relative group overflow-hidden"
                    onClick={() => handlePreview(file)}
                >
                    <div className="absolute inset-0 flex items-center justify-center text-gray-500">
                        <Image size={24} />
                    </div>
                    {/* Placeholder until loaded? Actual thumbnails require pulling.
                        For now, we just show formatted tiles. 
                    */}
                    <div className="absolute bottom-0 left-0 right-0 bg-black/60 p-1 truncate text-[10px] text-gray-300">
                        {file.name}
                    </div>
                </div>
            ))}
        </div>
    );

    const renderAlbums = () => (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(140px,1fr))] gap-4">
            {Object.entries(albums).map(([name, files]) => (
                <div key={name} className="flex flex-col bg-[#1a1a1f] rounded-xl p-3 hover:bg-[#27272a] transition">
                    <div className="aspect-video bg-[#0f0f12] rounded mb-2 flex items-center justify-center relative overflow-hidden">
                        <span className="text-gray-600 font-bold text-xl">{files.length}</span>
                        {/* Could preview first image here if we pulled it */}
                    </div>
                    <span className="font-medium text-sm truncate" title={name}>{name}</span>
                    <span className="text-xs text-gray-500">{files.length} items</span>
                </div>
            ))}
        </div>
    );

    return (
        <div className="h-full flex flex-col bg-[#0f0f12] text-white">
            {/* Header */}
            <div className="flex items-center gap-4 p-4 border-b border-[#27272a]">
                <div className="flex bg-[#1a1a1f] rounded-lg p-1">
                    <button
                        onClick={() => setTab("recent")}
                        className={`px-4 py-1.5 rounded-md text-sm font-medium transition ${tab === 'recent' ? 'bg-[#2ca9bc] text-white' : 'text-gray-400 hover:text-white'}`}
                    >
                        Recent
                    </button>
                    <button
                        onClick={() => setTab("albums")}
                        className={`px-4 py-1.5 rounded-md text-sm font-medium transition ${tab === 'albums' ? 'bg-[#2ca9bc] text-white' : 'text-gray-400 hover:text-white'}`}
                    >
                        Albums
                    </button>
                </div>
                <div className="ml-auto text-xs text-gray-500">
                    {media.length} photos found
                </div>
            </div>

            {/* Content */}
            <div className="flex-1 overflow-y-auto p-4">
                {loading ? (
                    <div className="flex items-center justify-center h-full">
                        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
                    </div>
                ) : (
                    tab === "recent" ? renderRecent() : renderAlbums()
                )}
            </div>

            {/* Preview Modal */}
            {preview && (
                <div className="fixed inset-0 bg-black/90 z-[60] flex flex-col animate-fade-in">
                    <div className="flex justify-between items-center p-4">
                        <h3 className="text-sm font-mono truncate max-w-md">{preview.file.name}</h3>
                        <div className="flex gap-4">
                            <button onClick={() => handleDownload(preview.file)} className="p-2 hover:bg-white/10 rounded-full">
                                <Download size={20} />
                            </button>
                            <button onClick={() => setPreview(null)} className="p-2 hover:bg-white/10 rounded-full">
                                <X size={20} />
                            </button>
                        </div>
                    </div>

                    <div className="flex-1 flex items-center justify-center p-4 overflow-hidden">
                        {previewLoading ? (
                            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-white"></div>
                        ) : (
                            <img
                                key={preview.src}
                                src={preview.src}
                                className="max-w-full max-h-full object-contain shadow-2xl"
                                alt="Preview"
                            />
                        )}
                    </div>
                </div>
            )}
        </div>
    );
}
