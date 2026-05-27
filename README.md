# DroidOps

**Professional Android Device Manager** - A modern, cross-platform desktop GUI for Android Debug Bridge (ADB)

Built with Tauri, React, and Tailwind CSS, DroidOps simplifies Android device management through an intuitive interface for operations that typically require command-line interaction.

![Version](https://img.shields.io/badge/version-0.1.3-blue)
![License](https://img.shields.io/badge/license-MIT-green)

---

## ✨ Features

- **📊 Device Dashboard** - Real-time device info including model, Android version, and battery status
- **📱 App Manager** - List, install, and uninstall applications with ease
- **📁 File Explorer** - Browse and manage device files with drag-and-drop support
- **🖼️ Gallery** - View device photos organized by albums with thumbnail caching
- **⚡ Fastboot Tools** - Dedicated interface for bootloader operations
- **📦 Sideloading** - Streamlined APK installation via drag-and-drop
- **💻 ADB Shell** - Integrated terminal for direct ADB command execution

## 🎨 Technology Stack

| Layer | Technology |
|-------|-----------|
| **Core** | Tauri v2 (Rust) |
| **Frontend** | React + Vite |
| **Styling** | Tailwind CSS v4 |
| **Icons** | Lucide React |
| **State** | React Hooks |

---

## 🚀 Getting Started

### Prerequisites

Ensure you have the following installed:

- **Node.js** (LTS recommended)
- **Rust & Cargo** (for Tauri)
- **ADB** (Android Debug Bridge) - Added to system PATH
- **Visual Studio C++ Build Tools** (Windows only)

### Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/bbinxx/droidops.git
   cd droidops
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

3. **Run in development mode:**
   ```bash
   npm run tauri dev
   ```

### Building for Production

```bash
npm run tauri build
```

Build artifacts will be in `src-tauri/target/release/bundle/`

---

## 📁 Project Architecture

### Modular Structure

```
src/
├── components/         # Reusable UI components
│   ├── DeviceSelector.jsx
│   ├── FileExplorer.jsx  
│   ├── Gallery.jsx
│   └── Sidebar.jsx
├── pages/             # Page-level components
│   ├── Dashboard.jsx
│   ├── AppManager.jsx
│   ├── ShellPage.jsx
│   ├── FastbootPage.jsx
│   └── SideloadPage.jsx
├── lib/               # Core functionality
│   ├── adb.js         # ADB command wrappers
│   └── cache.js       # File caching utilities
├── data/              # Configuration & strings
│   ├── strings.js     # All UI text/labels (i18n-ready)
│   └── config.js      # App configuration & constants
├── hooks/             # Custom React hooks
│   └── useStrings.js  # String management hook
├── utils/             # Utility functions
│   └── dialog.js      # Standardized dialogs
└── App.jsx           # Main application
```

### Data-Driven Design

DroidOps uses a **fully modular, data-driven architecture** making it easy to:
- 🌐 Add internationalization (i18n)
- 🔧 Modify configuration without touching code
- 🎨 Maintain consistent UI/UX
- 🧩 Extend features easily

#### Strings Management

All UI text is centralized in `src/data/strings.js`:

```javascript
import { STRINGS, formatString } from '../data/strings';

// Direct access
const title = STRINGS.dashboard.title;

// With placeholders
const message = formatString(STRINGS.apps.confirmUninstall, { app: 'MyApp' });
```

**String Categories:**
- `app` - Application metadata
- `navigation` - Menu labels
- `device` - Device-related strings
- `dashboard`, `apps`, `files`, `gallery` - Page-specific strings
- `errors` - Error messages
- `common` - Common UI strings

#### Configuration

App-wide settings in `src/data/config.js`:

```javascript
import { APP_CONFIG, ROUTES, DEVICE_STATES } from '../data/config';

// Use configuration
const theme = APP_CONFIG.ui.defaultTheme;

// Use route constants
navigate(ROUTES.DASHBOARD);

// Use device states
if (device.state === DEVICE_STATES.OFFLINE) { ... }
```

#### Standardized Dialogs

```javascript
import { confirmDelete, confirmUninstall, alertDialog } from '../utils/dialog';

// Confirm actions
if (confirmDelete(fileName)) {
  deleteFile();
}

// Show alerts
alertDialog(STRINGS.files.uploadSuccess);
```

---

## 🎨 Styling Conventions

- **Framework**: Tailwind CSS
- **Theme**: Dark by default
- **Color Palette**:
  - Primary: `#2ca9bc`
  - Background: `#0f0f12`
  - Surface: `#1a1a1f`
  - Border: `#27272a`

---

## 🌐 Development Guide

### Component Guidelines

#### 1. Import Order
```javascript
// 1. React & third-party
import React, { useState } from 'react';
import { Icon } from 'lucide-react';

// 2. Data & config
import { STRINGS } from '../data/strings';
import { APP_CONFIG } from '../data/config';

// 3. Utilities & hooks
import { useStrings } from '../hooks/useStrings';
import { confirmDialog } from '../utils/dialog';

// 4. Local imports
import { myFunction } from '../lib/myLib';
```

#### 2. Use Constants
```javascript
// ❌ Bad
if (activeTab === "dashboard") { ... }

// ✅ Good
import { ROUTES } from '../data/config';
if (activeTab === ROUTES.DASHBOARD) { ... }
```

#### 3. Use Centralized Strings
```javascript
// ❌ Bad
<button>Delete</button>

// ✅ Good
<button>{STRINGS.files.delete}</button>
```

### Adding New Features

#### Adding a New Page

1. Create page component in `src/pages/`
2. Add route to `ROUTES` in `config.js`
3. Add strings to appropriate section in `strings.js`
4. Register route in `App.jsx`
5. Add navigation item to `Sidebar.jsx`

#### Adding New Strings

1. Add to `src/data/strings.js`:
```javascript
export const STRINGS = {
  myFeature: {
    title: "My Feature",
    description: "Description here",
    action: "Action button label"
  }
};
```

2. Use in component:
```javascript
import { STRINGS } from '../data/strings';
<h1>{STRINGS.myFeature.title}</h1>
```

#### Adding Configuration

1. Add to `src/data/config.js`:
```javascript
export const APP_CONFIG = {
  myFeature: {
    setting1: true,
    setting2: "value"
  }
};
```

2. Use in code:
```javascript
import { APP_CONFIG } from '../data/config';
if (APP_CONFIG.myFeature.setting1) { ... }
```

### Best Practices

1. ✅ **Keep components focused** - Single responsibility
2. ✅ **Use data files** - No hardcoded strings/config
3. ✅ **Leverage hooks** - Extract reusable logic
4. ✅ **Handle errors gracefully** - Use standardized dialogs
5. ✅ **Document new features** - Update documentation

### Migration Checklist

When updating existing code:
- [ ] Replace hardcoded strings with `STRINGS` references
- [ ] Replace magic values with `APP_CONFIG` constants
- [ ] Use `ROUTES` for navigation logic
- [ ] Use dialog utilities instead of raw `window.confirm/alert`
- [ ] Extract reusable logic into hooks

---

## 🤝 Contributing

Contributions are welcome! Please follow the modular architecture and coding standards outlined above.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Follow the component guidelines and use data-driven design
4. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
5. Push to the branch (`git push origin feature/AmazingFeature`)
6. Open a Pull Request

---

## 📝 License

This project is licensed under the MIT License.

---

## 📚 Resources

- [Tauri Documentation](https://tauri.app/)
- [React Documentation](https://react.dev/)
- [Tailwind CSS](https://tailwindcss.com/)
- [ADB Documentation](https://developer.android.com/tools/adb)

---

## 🙏 Acknowledgments

Built with ❤️ using modern web technologies and Rust for a fast, secure, and beautiful desktop experience.
