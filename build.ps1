# CSV Tool GUI - Simplified Build Script
# This is the ONLY script you need to run!

param(
    [switch]$SkipIconCheck,
    [switch]$Clean
)

$ErrorActionPreference = "Stop"

# Colors for output
function Write-Info { Write-Host $args -ForegroundColor Cyan }
function Write-Success { Write-Host $args -ForegroundColor Green }
function Write-Warning { Write-Host $args -ForegroundColor Yellow }
function Write-Error { Write-Host $args -ForegroundColor Red }

Write-Info "========================================"
Write-Info "  CSV Tool GUI - Build Script"
Write-Info "========================================"
Write-Host ""

# Change to script directory
$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptPath
Write-Info "[INFO] Working directory: $PWD"
Write-Host ""

# Clean if requested
if ($Clean) {
    Write-Warning "[CLEAN] Cleaning build artifacts..."
    cargo clean 2>$null
    if (Test-Path "tauri") {
        Set-Location tauri
        cargo clean 2>$null
        Set-Location ..
    }
    if (Test-Path "frontend\dist") {
        Remove-Item -Recurse -Force "frontend\dist" -ErrorAction SilentlyContinue
    }
    Write-Success "[OK] Clean completed"
    Write-Host ""
}

# Check Rust
Write-Info "[CHECK] Checking Rust..."

# Try to find Rust in common locations
$rustPaths = @(
    "rustc",  # Try PATH first
    "$env:USERPROFILE\.cargo\bin\rustc.exe",
    "$env:LOCALAPPDATA\Programs\rust\bin\rustc.exe",
    "C:\Users\$env:USERNAME\.cargo\bin\rustc.exe"
)

$rustcPath = $null
foreach ($path in $rustPaths) {
    if ($path -eq "rustc") {
        # Try command directly
        $test = rustc --version 2>&1
        if ($LASTEXITCODE -eq 0 -and $test) {
            $rustcPath = "rustc"
            break
        }
    } else {
        if (Test-Path $path) {
            $rustcPath = $path
            break
        }
    }
}

if ($rustcPath) {
    # Add to PATH if found in non-standard location
    if ($rustcPath -ne "rustc") {
        $cargoBin = Split-Path -Parent $rustcPath
        if ($env:Path -notlike "*$cargoBin*") {
            $env:Path += ";$cargoBin"
            Write-Info "[INFO] Added Rust to PATH for this session"
        }
    }
    
    # Get version
    if ($rustcPath -eq "rustc") {
        $rustVersion = rustc --version 2>&1 | Select-Object -First 1
    } else {
        $rustVersion = & $rustcPath --version 2>&1 | Select-Object -First 1
    }
    
    Write-Success "[OK] $rustVersion"
} else {
    Write-Error "[ERROR] Rust not found"
    Write-Warning "Rust is installed but not in PATH"
    Write-Host ""
    Write-Warning "Quick fix - Run this command:"
    Write-Host "  `$env:Path += ';$env:USERPROFILE\.cargo\bin'" -ForegroundColor Gray
    Write-Host ""
    Write-Warning "Or restart PowerShell to reload PATH"
    Write-Host ""
    Write-Warning "If Rust is not installed, install from: https://rustup.rs/"
    Write-Host "Or run: .\install_rust.ps1"
    exit 1
}

# Check Node.js
Write-Info "[CHECK] Checking Node.js..."

# Try to find Node.js
$nodePaths = @(
    "node",  # Try PATH first
    "$env:ProgramFiles\nodejs\node.exe",
    "${env:ProgramFiles(x86)}\nodejs\node.exe"
)

$nodePath = $null
foreach ($path in $nodePaths) {
    if ($path -eq "node") {
        $test = node --version 2>&1
        if ($LASTEXITCODE -eq 0 -and $test) {
            $nodePath = "node"
            break
        }
    } else {
        if (Test-Path $path) {
            $nodePath = $path
            break
        }
    }
}

if ($nodePath) {
    # Add to PATH if found in non-standard location
    if ($nodePath -ne "node") {
        $nodeBin = Split-Path -Parent $nodePath
        if ($env:Path -notlike "*$nodeBin*") {
            $env:Path += ";$nodeBin"
            Write-Info "[INFO] Added Node.js to PATH for this session"
        }
    }
    
    # Get version
    if ($nodePath -eq "node") {
        $nodeVersion = node --version 2>&1 | Select-Object -First 1
    } else {
        $nodeVersion = & $nodePath --version 2>&1 | Select-Object -First 1
    }
    
    Write-Success "[OK] $nodeVersion"
} else {
    Write-Error "[ERROR] Node.js not found"
    Write-Warning "Please install Node.js from: https://nodejs.org/"
    exit 1
}

# Check directories
Write-Info "[CHECK] Checking project structure..."
if (-not (Test-Path "frontend")) {
    Write-Error "[ERROR] frontend directory not found"
    exit 1
}
if (-not (Test-Path "tauri")) {
    Write-Error "[ERROR] tauri directory not found"
    exit 1
}
Write-Success "[OK] Project structure valid"
Write-Host ""

# Check icon file
if (-not $SkipIconCheck) {
    Write-Info "[CHECK] Checking icon file..."
    if (-not (Test-Path "tauri\icons\icon.ico")) {
        Write-Warning "[WARNING] Icon file not found: tauri\icons\icon.ico"
        Write-Host ""
        Write-Warning "Tauri Windows build REQUIRES an icon file!"
        Write-Host ""
        Write-Host "Quick fix:" -ForegroundColor Yellow
        Write-Host "  1. Visit: https://convertio.co/zh/png-ico/" -ForegroundColor Gray
        Write-Host "  2. Upload any PNG image (256x256+ recommended)" -ForegroundColor Gray
        Write-Host "  3. Download as ICO format" -ForegroundColor Gray
        Write-Host "  4. Save to: tauri\icons\icon.ico" -ForegroundColor Gray
        Write-Host ""
        Write-Host "Or use Tauri CLI:" -ForegroundColor Yellow
        Write-Host "  cd tauri" -ForegroundColor Gray
        Write-Host "  cargo tauri icon path/to/icon.png" -ForegroundColor Gray
        Write-Host ""
        $continue = Read-Host "Continue build anyway? (y/N)"
        if ($continue -ne "y" -and $continue -ne "Y") {
            Write-Info "Build cancelled. Create icon file and try again."
            exit 0
        }
        Write-Host ""
    } else {
        Write-Success "[OK] Icon file found"
    }
}

# Install frontend dependencies
Write-Info "[STEP 1/4] Installing frontend dependencies..."
if (-not (Test-Path "frontend\node_modules")) {
    Set-Location frontend
    npm install
    if ($LASTEXITCODE -ne 0) {
        Write-Error "[ERROR] Frontend dependency installation failed"
        Set-Location ..
        exit 1
    }
    Set-Location ..
    Write-Success "[OK] Frontend dependencies installed"
} else {
    Write-Info "[SKIP] Frontend dependencies already installed"
}
Write-Host ""

# Check/Install Tauri CLI
Write-Info "[STEP 2/4] Checking Tauri CLI..."
$tauriCheck = cargo tauri --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Warning "[WARN] Tauri CLI not found, installing..."
    Write-Info "This may take 5-15 minutes..."
    cargo install tauri-cli --version "^1.5" --locked
    if ($LASTEXITCODE -ne 0) {
        Write-Error "[ERROR] Tauri CLI installation failed"
        Write-Warning "Possible reasons:"
        Write-Warning "  1. Rust version too old (need 1.84+)"
        Write-Warning "  2. Network issue"
        Write-Warning "  3. Insufficient disk space"
        exit 1
    }
    Write-Success "[OK] Tauri CLI installed"
} else {
    Write-Success "[OK] Tauri CLI found"
}
Write-Host ""

# Build frontend
Write-Info "[STEP 3/4] Building frontend..."
Set-Location frontend
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Error "[ERROR] Frontend build failed"
    Set-Location ..
    exit 1
}
Set-Location ..
Write-Success "[OK] Frontend build completed"
Write-Host ""

# Build Tauri
Write-Info "[STEP 4/4] Building Tauri app (Release mode)..."
Write-Info "This may take 10-30 minutes on first build..."
Write-Host ""
Set-Location tauri
cargo tauri build
$buildResult = $LASTEXITCODE
Set-Location ..

if ($buildResult -ne 0) {
    Write-Host ""
    Write-Error "========================================"
    Write-Error "  BUILD FAILED"
    Write-Error "========================================"
    Write-Host ""
    Write-Warning "Error code: $buildResult"
    Write-Host ""
    Write-Warning "Common issues:"
    Write-Host "  1. Missing icon file: tauri\icons\icon.ico" -ForegroundColor Gray
    Write-Host "  2. Missing Visual C++ Build Tools" -ForegroundColor Gray
    Write-Host "     Download: https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor Gray
    Write-Host "  3. Insufficient disk space (need 5GB+)" -ForegroundColor Gray
    Write-Host "  4. Network issue (dependency download failed)" -ForegroundColor Gray
    Write-Host "  5. Rust version too old (need 1.84+)" -ForegroundColor Gray
    Write-Host ""
    Write-Info "Run with -Clean to clean build artifacts: .\build.ps1 -Clean"
    exit 1
}

Write-Host ""
Write-Success "========================================"
Write-Success "  BUILD SUCCESSFUL!"
Write-Success "========================================"
Write-Host ""

# Check output files
Write-Info "[RESULT] Build output:"
Write-Host ""

if (Test-Path "tauri\target\release\csv-tool.exe") {
    $exeFile = Get-Item "tauri\target\release\csv-tool.exe"
    Write-Success "[OK] EXE file:"
    Write-Host "    $($exeFile.FullName)" -ForegroundColor Gray
    Write-Host "    Size: $([math]::Round($exeFile.Length / 1MB, 2)) MB" -ForegroundColor Gray
    Write-Host ""
} else {
    Write-Warning "[WARN] EXE file not found at expected location"
    Write-Host ""
}

$installerFiles = Get-ChildItem "tauri\target\release\bundle\msi\*.msi" -ErrorAction SilentlyContinue
if ($installerFiles) {
    Write-Success "[OK] Installer:"
    $installerFiles | ForEach-Object {
        Write-Host "    $($_.FullName)" -ForegroundColor Gray
        Write-Host "    Size: $([math]::Round($_.Length / 1MB, 2)) MB" -ForegroundColor Gray
    }
    Write-Host ""
} else {
    Write-Info "[INFO] Installer not found (this is OK, EXE is the main output)"
    Write-Host ""
}

Write-Success "You can now double-click the EXE file to run the application!"
Write-Host ""
