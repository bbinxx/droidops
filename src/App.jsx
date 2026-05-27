import React, { useState } from "react";
import { Sidebar } from "./components/Sidebar";
import { DeviceSelector } from "./components/DeviceSelector";
import { Dashboard } from "./pages/Dashboard";
import { AppManager } from "./pages/AppManager";
import { ShellPage } from "./pages/ShellPage";
import { FastbootPage } from "./pages/FastbootPage";
import { SideloadPage } from "./pages/SideloadPage";
import { FileExplorer } from "./components/FileExplorer";
import { Gallery } from "./components/Gallery";
import "./index.css";

function App() {
  const [activeTab, setActiveTab] = useState("dashboard");
  const [selectedDevice, setSelectedDevice] = useState(null);

  const renderContent = () => {
    switch (activeTab) {
      case "dashboard": return <Dashboard selectedDevice={selectedDevice} />;
      case "apps": return <AppManager selectedDevice={selectedDevice} />;
      case "files": return <FileExplorer selectedDevice={selectedDevice} />;
      case "gallery": return <Gallery selectedDevice={selectedDevice} />;
      case "terminal": return <ShellPage selectedDevice={selectedDevice} />;
      case "fastboot": return <FastbootPage selectedDevice={selectedDevice} />;
      case "sideload": return <SideloadPage selectedDevice={selectedDevice} />;
      default: return <Dashboard selectedDevice={selectedDevice} />;
    }
  };

  return (
    <div className="flex h-screen bg-[#0f0f12] text-white overflow-hidden font-sans selection:bg-blue-500/30">
      <Sidebar activeTab={activeTab} setActiveTab={setActiveTab} />

      <main className="flex-1 flex flex-col relative">
        {/* Top Bar / Header */}
        <header className="h-16 flex items-center justify-between px-8 border-b border-[#27272a] bg-[#0f0f12]/80 backdrop-blur-xl z-20">
          <h2 className="text-gray-400 font-medium capitalize">{activeTab.replace("-", " ")}</h2>

          {/* Device Selector in Top Right Corner */}
          <DeviceSelector
            selectedDevice={selectedDevice}
            onSelect={setSelectedDevice}
            className="ml-auto"
          />
        </header>

        {/* Page Content */}
        <div className="flex-1 overflow-auto relative z-10">
          {renderContent()}
        </div>
      </main>
    </div>
  );
}

export default App;
