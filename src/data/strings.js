// Note: App metadata (name, version) is in config.js (imported from package.json)
// This keeps it as single source of truth. Access via: import { APP_CONFIG } from './config'

export const STRINGS = {
    app: {
        title: "DroidOps", // Display name - also available as APP_CONFIG.app.displayName
        tagline: "Professional Android Device Manager"
    },

    navigation: {
        dashboard: "Dashboard",
        apps: "App Manager",
        files: "File Explorer",
        gallery: "Gallery",
        terminal: "Terminal",
        fastboot: "Fastboot",
        sideload: "Sideload"
    },

    device: {
        label: "Device:",
        noDevicesFound: "No Devices Found",
        refreshTooltip: "Refresh",
        connecting: "Connecting...",
        connected: "Connected",
        unauthorized: "Unauthorized",
        offline: "Offline"
    },

    dashboard: {
        title: "Device Overview",
        batteryLevel: "Battery Level",
        batteryStatus: "Status",
        model: "Model",
        androidVersion: "Android Version",
        charging: "Charging",
        discharging: "Discharging",
        notCharging: "Not Charging",
        full: "Full",
        unknown: "Unknown"
    },

    apps: {
        title: "Application Manager",
        installedApps: "Installed Applications",
        userApps: "User Apps",
        systemApps: "System Apps",
        allApps: "All Apps",
        uninstall: "Uninstall",
        clearData: "Clear Data",
        confirmUninstall: "Are you sure you want to uninstall {app}?",
        confirmClearData: "Are you sure you want to clear data for {app}?",
        uninstallSuccess: "Application uninstalled successfully",
        uninstallFailed: "Failed to uninstall application",
        clearDataSuccess: "Application data cleared",
        clearDataFailed: "Failed to clear data"
    },

    files: {
        title: "File Explorer",
        path: "Path:",
        newFolder: "New Folder",
        upload: "Upload",
        download: "Download",
        delete: "Delete",
        rename: "Rename",
        refresh: "Refresh",
        emptyFolder: "Empty Folder",
        confirmDelete: "Are you sure you want to delete {item}?",
        createFolderPrompt: "Enter folder name:",
        renamePrompt: "Enter new name:",
        deleteSuccess: "Item deleted",
        deleteFailed: "Failed to delete item",
        uploadSuccess: "Upload complete",
        uploadFailed: "Upload failed",
        downloadSuccess: "Download complete",
        downloadFailed: "Download failed",
        folderCreated: "Folder created",
        renamed: "Renamed successfully",
        actionFailed: "Action failed: {error}"
    },

    gallery: {
        title: "Gallery",
        recent: "Recent",
        albums: "Albums",
        photosFound: "{count} photos found",
        items: "{count} items",
        downloadComplete: "Download Complete",
        downloadFailed: "Download failed",
        previewFailed: "Failed to load image preview",
        mirrorScreen: "Mirror Screen"
    },

    terminal: {
        title: "ADB Shell",
        placeholder: "Type ADB command...",
        clear: "Clear",
        run: "Run"
    },

    fastboot: {
        title: "Fastboot Mode",
        noDevices: "No fastboot devices found",
        commands: "Available Commands",
        reboot: "Reboot",
        rebootBootloader: "Reboot to Bootloader",
        flash: "Flash Image",
        erase: "Erase Partition",
        format: "Format Partition"
    },

    sideload: {
        title: "Sideload APK",
        selectFile: "Select APK File",
        install: "Install",
        installing: "Installing...",
        installSuccess: "Installation complete",
        installFailed: "Installation failed"
    },

    errors: {
        noDeviceSelected: "No device selected",
        deviceNotFound: "Device not found",
        commandFailed: "Command failed",
        connectionLost: "Connection lost",
        permissionDenied: "Permission denied",
        unknownError: "Unknown error occurred"
    },

    common: {
        ok: "OK",
        cancel: "Cancel",
        yes: "Yes",
        no: "No",
        save: "Save",
        close: "Close",
        loading: "Loading...",
        search: "Search",
        filter: "Filter",
        browse: "Browse",
        apply: "Apply",
        reset: "Reset"
    }
};

// Helper function to replace placeholders in strings
export const formatString = (template, values = {}) => {
    return template.replace(/\{(\w+)\}/g, (match, key) => values[key] || match);
};
