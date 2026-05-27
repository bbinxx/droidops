import React, { useState, useEffect, useRef } from "react";
import {
    Folder, File, ArrowLeft, RefreshCw, Home, Download, Trash2,
    MoreVertical, Upload, FileImage, FileVideo, Music, FileText, Smartphone
} from "lucide-react";
import { listFiles, pushFile, pullFile, deleteFile, createDirectory, renameFile } from "../lib/adb";
import { getThumbnail } from "../lib/cache";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { listen } from "@tauri-apps/api/event";
import { open, save } from '@tauri-apps/plugin-dialog';

const FileThumbnail = ({ file, serial }) => {
    const [src, setSrc] = useState(null);
    const [loading, setLoading] = useState(true);
    const mounted = useRef(true);

    useEffect(() => {
        mounted.current = true;
        const load = async () => {
            const url = await getThumbnail(serial, file.path, file.name);
            if (mounted.current) {
                setSrc(url);
                setLoading(false);
            }
        };
        load();
        return () => { mounted.current = false; };
    }, [serial, file]);

    if (loading || !src) {
        return <FileImage className="text-purple-400" size={24} />;
    }

    return <img src={src} className="w-10 h-10 object-cover rounded shadow" />;
};

export function FileExplorer({ selectedDevice }) {
    const [path, setPath] = useState("/sdcard/");
    const [files, setFiles] = useState([]);
    const [loading, setLoading] = useState(false);
    const [selectedFile, setSelectedFile] = useState(null);
    const [contextMenu, setContextMenu] = useState(null);
    const [statusMsg, setStatusMsg] = useState("");
    const [refreshTrigger, setRefreshTrigger] = useState(0);
    const [modal, setModal] = useState({ open: false, type: "", value: "" });

    const fileListRef = useRef(null);

    // Refresh files when path or device changes
    useEffect(() => {
        if (!selectedDevice) return;
        fetchFiles();
    }, [selectedDevice, path, refreshTrigger]);

    // Setup File Drop Listener
    useEffect(() => {
        let unlisten = null;
        async function setupListener() {
            unlisten = await listen('tauri://file-drop', async (event) => {
                // event.payload is string[] of absolute paths
                if (selectedDevice && event.payload && event.payload.length > 0) {
                    handleFileUpload(event.payload);
                }
            });
        }
        setupListener();
        return () => {
            if (unlisten) unlisten();
        };
    }, [selectedDevice, path]);

    const handleCreateFolder = async () => {
        setModal({ open: true, type: "new-folder", value: "" });
    };

    const handleRename = () => {
        if (!contextMenu?.file) return;
        setModal({
            open: true,
            type: "rename",
            value: contextMenu.file.name,
            targetFile: contextMenu.file
        });
        setContextMenu(null);
    };

    const processModalAction = async () => {
        if (!modal.value.trim()) return;
        setLoading(true);
        try {
            if (modal.type === "new-folder") {
                const newPath = path.endsWith("/") ? path + modal.value : path + "/" + modal.value;
                await createDirectory(selectedDevice.serial, newPath);
                setStatusMsg("Folder created");
            } else if (modal.type === "rename") {
                const oldPath = modal.targetFile.path;
                // We assume file is in current 'path' but 'file.path' is absolute.
                // We need to calculate new absolute path.
                // oldPath: /sdcard/foo.txt
                // newPath: /sdcard/bar.txt
                const parentDir = oldPath.substring(0, oldPath.lastIndexOf('/'));
                const newPath = parentDir + "/" + modal.value;

                await renameFile(selectedDevice.serial, oldPath, newPath);
                setStatusMsg("Renamed");
            }
        } catch (e) {
            console.error(e);
            setStatusMsg("Action failed: " + e.message);
        }
        setRefreshTrigger(p => p + 1);
        setModal({ open: false, type: "", value: "" });
        setLoading(false);
    };

    const fetchFiles = async () => {
        setLoading(true);
        setStatusMsg("");
        try {
            const list = await listFiles(selectedDevice?.serial, path);
            // Sort: Directories first, then files. Alphabetical.
            list.sort((a, b) => {
                if (a.isDirectory === b.isDirectory) {
                    return a.name.localeCompare(b.name);
                }
                return a.isDirectory ? -1 : 1;
            });
            setFiles(list);
        } catch (error) {
            console.error(error);
            setStatusMsg("Failed to list files");
        } finally {
            setLoading(false);
        }
    };

    const handleFileUpload = async (localPaths) => {
        if (!selectedDevice) return;
        setStatusMsg(`Uploading ${localPaths.length} files...`);
        try {
            for (const localPath of localPaths) {
                // Determine destination filename? pushFile handles generic dest path usually
                // but adb push local remote_dir works fine.
                await pushFile(selectedDevice.serial, localPath, path);
            }
            setStatusMsg("Upload complete");
            setRefreshTrigger(p => p + 1);
        } catch (err) {
            console.error(err);
            setStatusMsg("Upload failed: " + err.message);
        }
    };

    const handleDownload = async (file) => {
        if (!file) return;
        try {
            // Open save dialog to pick location
            // If it's a folder, we might need a directory picker? 
            // save() is for files. open({ directory: true }) is for dirs.
            // ADB pull works recursively for dirs.
            // Let's assume user wants to save to a specific folder on PC.

            // For simplicity: If file, use save dialog. If directory, use open dialog (select folder).
            let targetPath = null;

            if (file.isDirectory) {
                const selected = await open({
                    directory: true,
                    multiple: false,
                    title: "Select Destination Folder"
                });
                if (selected) {
                    // pull /sdcard/foo to C:/Users/Bar/foo
                    // ADB pull src dest requires dest to be the parent dir or full path?
                    // adb pull /sdcard/foo C:/Users/Bar/ -> creates C:/Users/Bar/foo
                    targetPath = selected;
                }
            } else {
                targetPath = await save({
                    defaultPath: file.name,
                    title: "Save File"
                });
            }

            if (targetPath) {
                setStatusMsg(`Downloading ${file.name}...`);
                await pullFile(selectedDevice.serial, file.path, targetPath);
                setStatusMsg("Download complete");
            }
        } catch (err) {
            console.error(err);
            setStatusMsg("Download failed");
        }
        setContextMenu(null);
    };

    const handleDelete = async (file) => {
        if (!file) return;
        if (!confirm(`Are you sure you want to delete ${file.name}?`)) {
            setContextMenu(null);
            return;
        }

        try {
            setStatusMsg(`Deleting ${file.name}...`);
            await deleteFile(selectedDevice.serial, file.path);
            setStatusMsg("Deleted");
            setRefreshTrigger(p => p + 1);
        } catch (err) {
            setStatusMsg("Delete failed");
        }
        setContextMenu(null);
    };

    const navigateUp = () => {
        if (path === "/") return;
        const parts = path.split("/").filter(Boolean);
        parts.pop();
        const newPath = "/" + parts.join("/") + (parts.length ? "/" : "");
        setPath(newPath || "/"); // ensure root is /
    };

    const openFolder = (folderName) => {
        setPath(prev => prev.endsWith("/") ? prev + folderName + "/" : prev + "/" + folderName + "/");
    };

    const getFileIcon = (file) => {
        if (file.isDirectory) return <Folder className="text-yellow-400 fill-yellow-400/20" size={24} />;
        const ext = file.name.split(".").pop().toLowerCase();

        if (['jpg', 'png', 'jpeg', 'gif', 'webp'].includes(ext)) {
            // Render Async Thumbnail
            return <FileThumbnail file={file} serial={selectedDevice.serial} />;
        }

        if (['mp4', 'mkv', 'webm'].includes(ext)) return <FileVideo className="text-red-400" size={24} />;
        if (['mp3', 'wav', 'ogg'].includes(ext)) return <Music className="text-green-400" size={24} />;
        if (['txt', 'log', 'xml', 'json'].includes(ext)) return <FileText className="text-gray-400" size={24} />;
        if (['apk'].includes(ext)) return <Smartphone className="text-green-500" size={24} />;
        return <File className="text-blue-400" size={24} />;
    };

    if (!selectedDevice) {
        return (
            <div className="flex items-center justify-center h-full text-gray-500">
                <p>Select a device to browse files</p>
            </div>
        );
    }

    return (
        <div className="h-full flex flex-col bg-[#0f0f12] text-gray-200" onClick={() => setContextMenu(null)}>
            {/* Toolbar */}
            <div className="flex items-center gap-2 p-4 border-b border-[#27272a] bg-[#1a1a1f]/50">
                <button onClick={navigateUp} disabled={path === "/" || path === "/sdcard/"} className="p-2 hover:bg-[#27272a] rounded disabled:opacity-30">
                    <ArrowLeft size={18} />
                </button>
                <button onClick={() => setPath("/sdcard/")} className="p-2 hover:bg-[#27272a] rounded">
                    <Home size={18} />
                </button>

                <div className="flex-1 bg-[#27272a] rounded px-3 py-1.5 text-sm font-mono truncate flex items-center">
                    <span className="text-gray-500 mr-2">path:</span>
                    {path}
                </div>

                <button onClick={handleCreateFolder} className="p-2 hover:bg-[#27272a] rounded text-blue-400" title="New Folder">
                    <Folder size={18} />
                    {/* <span className="text-xs ml-1 font-bold">+</span> */}
                </button>

                <div className="flex items-center text-xs text-blue-400">
                    {statusMsg && <span className="animate-pulse mr-4">{statusMsg}</span>}
                </div>

                <button onClick={() => setRefreshTrigger(p => p + 1)} className="p-2 hover:bg-[#27272a] rounded">
                    <RefreshCw size={18} className={loading ? "animate-spin" : ""} />
                </button>
            </div>

            {/* File List */}
            <div ref={fileListRef} className="flex-1 overflow-y-auto p-4 content-start">
                {files.length === 0 && !loading && (
                    <div className="text-center text-gray-500 mt-10">Empty Folder</div>
                )}

                <div className="grid grid-cols-[repeat(auto-fill,minmax(120px,1fr))] gap-4">
                    {files.map((file) => (
                        <div
                            key={file.name}
                            className={`
                                group flex flex-col items-center justify-start p-3 rounded-xl cursor-pointer text-center
                                transition-all hover:bg-[#27272a] relative
                                ${selectedFile?.name === file.name ? "bg-[#2ca9bc]/20 ring-1 ring-[#2ca9bc]" : ""}
                            `}
                            onClick={(e) => {
                                e.stopPropagation();
                                setSelectedFile(file);
                            }}
                            onDoubleClick={(e) => {
                                e.stopPropagation();
                                if (file.isDirectory) openFolder(file.name);
                            }}
                            onContextMenu={(e) => {
                                e.preventDefault();
                                e.stopPropagation();
                                setSelectedFile(file);
                                setContextMenu({ x: e.clientX, y: e.clientY, file });
                            }}
                        >
                            {/* Icon */}
                            <div className="mb-3 transition-transform group-hover:scale-110">
                                {getFileIcon(file)}
                            </div>

                            {/* Name */}
                            <span className="text-xs w-full truncate px-1 text-gray-300 font-medium">
                                {file.name}
                            </span>

                            {/* Details (Size/Date) - Hover only? or minimal */}
                            {!file.isDirectory && (
                                <span className="text-[10px] text-gray-500 mt-0.5">
                                    {file.size}
                                </span>
                            )}
                        </div>
                    ))}
                </div>
            </div>

            {/* Drag Drop Hint Overlay */}
            <div className="absolute inset-0 pointer-events-none flex items-center justify-center opacity-0 transition-opacity bg-black/50 z-50">
                <div className="bg-[#1a1a1f] p-6 rounded-xl border border-dashed border-blue-500 flex flex-col items-center">
                    <Upload className="text-blue-500 mb-2" size={48} />
                    <h3 className="text-xl font-bold">Drop files to upload</h3>
                </div>
            </div>

            {/* Context Menu */}
            {contextMenu && (
                <div
                    className="fixed bg-[#1a1a1f] border border-[#27272a] rounded-lg shadow-xl z-50 py-1 min-w-[160px]"
                    style={{ top: contextMenu.y, left: contextMenu.x }}
                    onClick={(e) => e.stopPropagation()}
                >
                    <div className="px-3 py-2 border-b border-[#27272a] mb-1">
                        <p className="text-xs font-medium text-gray-400 truncate max-w-[140px]">{contextMenu.file.name}</p>
                    </div>
                    <button
                        className="w-full text-left px-4 py-2 hover:bg-[#27272a] flex items-center gap-2 text-sm"
                        onClick={() => handleDownload(contextMenu.file)}
                    >
                        <Download size={14} /> Download
                    </button>
                    <button
                        className="w-full text-left px-4 py-2 hover:bg-[#27272a] flex items-center gap-2 text-sm"
                        onClick={handleRename}
                    >
                        <FileText size={14} /> Rename
                    </button>
                    <button
                        className="w-full text-left px-4 py-2 hover:bg-[#27272a] flex items-center gap-2 text-sm text-red-400 hover:text-red-300"
                        onClick={() => handleDelete(contextMenu.file)}
                    >
                        <Trash2 size={14} /> Delete
                    </button>
                    {/* Rename could be added here */}
                </div>
            )}
            {/* Modal */}
            {modal.open && (
                <div className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
                    <div className="bg-[#1a1a1f] border border-[#27272a] p-6 rounded-2xl w-full max-w-sm shadow-2xl">
                        <h3 className="text-lg font-bold mb-4">
                            {modal.type === "new-folder" ? "New Folder" : "Rename File"}
                        </h3>
                        <input
                            autoFocus
                            type="text"
                            className="w-full bg-[#0f0f12] border border-[#27272a] rounded-lg px-4 py-2 text-white outline-none focus:border-blue-500 mb-6"
                            value={modal.value}
                            onChange={e => setModal({ ...modal, value: e.target.value })}
                            onKeyDown={e => e.key === "Enter" && processModalAction()}
                        />
                        <div className="flex justify-end gap-3">
                            <button
                                onClick={() => setModal({ ...modal, open: false })}
                                className="px-4 py-2 text-sm text-gray-400 hover:text-white"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={processModalAction}
                                className="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium"
                            >
                                {modal.type === "new-folder" ? "Create" : "Rename"}
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
