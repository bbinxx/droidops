# =============================================================================
# DroidOps — Local Build Script (Windows PowerShell)
# Usage:
#   .\scripts\build.ps1              — build all targets (msi + nsis)
#   .\scripts\build.ps1 -Msi        — Windows Installer (.msi) only
#   .\scripts\build.ps1 -Nsis       — NSIS installer (.exe) only
#   .\scripts\build.ps1 -Debug      — debug build (faster, unoptimized)
# =============================================================================

param(
    [switch]$Msi,
    [switch]$Nsis,
    [switch]$Debug,
    [switch]$Help
)

# ── Helpers ──────────────────────────────────────────────────────────────────
function Info    { param($msg) Write-Host "[build] $msg" -ForegroundColor Cyan }
function Success { param($msg) Write-Host "[OK]    $msg" -ForegroundColor Green }
function Warn    { param($msg) Write-Host "[!]     $msg" -ForegroundColor Yellow }
function Fail    { param($msg) Write-Host "[ERR]   $msg" -ForegroundColor Red; exit 1 }

if ($Help) {
    Write-Host "Usage: .\scripts\build.ps1 [-Msi] [-Nsis] [-Debug]"
    exit 0
}

# ── Header ───────────────────────────────────────────────────────────────────
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root

$pkg = Get-Content "package.json" | ConvertFrom-Json
$version = $pkg.version

Write-Host ""
Write-Host "══════════════════════════════════════════" -ForegroundColor Magenta
Write-Host "   DroidOps Build Script  v$version" -ForegroundColor Bold
Write-Host "══════════════════════════════════════════" -ForegroundColor Magenta
Write-Host ""

# ── Prerequisite checks ──────────────────────────────────────────────────────
Info "Checking prerequisites..."

if (-not (Get-Command node -ErrorAction SilentlyContinue)) { Fail "Node.js not found. Install from https://nodejs.org" }
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { Fail "Rust not found. Install from https://rustup.rs" }
if (-not (Get-Command npm -ErrorAction SilentlyContinue)) { Fail "npm not found." }

$nodeVer = (node --version).Trim()
$rustVer = (rustc --version).Split(" ")[1]
Success "Node.js $nodeVer  |  Rust $rustVer"

# ── Install npm deps ─────────────────────────────────────────────────────────
Info "Installing npm dependencies..."
npm ci 2>$null
if ($LASTEXITCODE -ne 0) { npm install }
Success "npm dependencies ready."

# ── Build args ───────────────────────────────────────────────────────────────
$bundleArgs = ""
if ($Msi)  { $bundleArgs = "--bundles msi" }
if ($Nsis) { $bundleArgs = "--bundles nsis" }

$buildFlag = if ($Debug) { "--debug" } else { "" }

# ── Run Build ────────────────────────────────────────────────────────────────
Write-Host ""
if ($Debug) {
    Info "Building DEBUG binary (fast, no optimizations)..."
} else {
    Info "Building RELEASE binary for Windows..."
}

$cmd = "npm run tauri build -- $buildFlag $bundleArgs".Trim()
Info "Running: $cmd"
Write-Host ""

Invoke-Expression $cmd
if ($LASTEXITCODE -ne 0) { Fail "Build failed with exit code $LASTEXITCODE" }

# ── Report artifacts ─────────────────────────────────────────────────────────
Write-Host ""
Success "Build complete!"
Info "Artifacts:"

$bundleDir = if ($Debug) {
    "src-tauri\target\debug\bundle"
} else {
    "src-tauri\target\release\bundle"
}

if (Test-Path $bundleDir) {
    Get-ChildItem $bundleDir -Recurse -Include "*.msi","*.exe" | ForEach-Object {
        $size = [math]::Round($_.Length / 1MB, 1)
        Write-Host "  -> $($_.FullName)  (${size} MB)" -ForegroundColor Green
    }
} else {
    Warn "Bundle dir not found: $bundleDir"
}

Write-Host ""
