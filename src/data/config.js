import pkg from '../../package.json';

export const APP_CONFIG = {
    // App Metadata (from package.json)
    app: {
        name: "DroidOps",
        displayName: "DroidOps",
        version: pkg.version,
        author: "bbinxx",
        repository: "https://github.com/bbinxx/droidops"
    },

    version: pkg.version,
    name: "DroidOps",

    // UI Configuration
    ui: {
        sidebarWidth: 240,
        headerHeight: 64,
        defaultTheme: "dark"
    },

    // File Explorer Settings
    fileExplorer: {
        defaultPath: "/sdcard/",
        showHiddenFiles: false,
        gridView: true,
        sortBy: "name" // name, size, date
    },

    // Gallery Settings
    gallery: {
        thumbnailSize: 120,
        previewQuality: "high",
        cacheEnabled: true,
        cachePath: "thumbnails"
    },

    // Terminal Settings
    terminal: {
        maxHistoryLines: 1000,
        fontSize: 14,
        fontFamily: "monospace"
    },

    // Device Settings
    device: {
        refreshInterval: 5000, // ms
        autoConnect: true,
        trackDevices: true
    },

    // App Manager Settings
    appManager: {
        defaultFilter: "user", // user, system, all
        showSystemApps: false,
        sortBy: "name"
    }
};

export const ROUTES = {
    DASHBOARD: "dashboard",
    APPS: "apps",
    FILES: "files",
    GALLERY: "gallery",
    TERMINAL: "terminal",
    FASTBOOT: "fastboot",
    SIDELOAD: "sideload"
};

export const DEVICE_STATES = {
    DEVICE: "device",
    OFFLINE: "offline",
    UNAUTHORIZED: "unauthorized",
    FASTBOOT: "fastboot"
};

export const FILE_EXTENSIONS = {
    images: ['jpg', 'png', 'jpeg', 'gif', 'webp', 'bmp'],
    videos: ['mp4', 'mkv', 'webm', 'avi', 'mov'],
    audio: ['mp3', 'wav', 'ogg', 'flac', 'm4a'],
    documents: ['txt', 'pdf', 'doc', 'docx', 'xml', 'json'],
    archives: ['zip', 'rar', '7z', 'tar', 'gz'],
    apk: ['apk']
};
