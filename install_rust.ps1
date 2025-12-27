# Install Rust - PowerShell Script

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Rust Installation Helper" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if Rust is already installed
Write-Host "[CHECK] Checking for existing Rust installation..." -ForegroundColor Yellow

$rustcPath = "$env:USERPROFILE\.cargo\bin\rustc.exe"
if (Test-Path $rustcPath) {
    Write-Host "[INFO] Rust found at: $rustcPath" -ForegroundColor Green
    & $rustcPath --version
    Write-Host ""
    Write-Host "[INFO] Rust is installed but may not be in PATH" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Solution 1: Restart PowerShell" -ForegroundColor Cyan
    Write-Host "Solution 2: Add to PATH manually:" -ForegroundColor Cyan
    Write-Host "  [Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ';$env:USERPROFILE\.cargo\bin', 'User')" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Solution 3: Run this command:" -ForegroundColor Cyan
    Write-Host "  `$env:Path += ';$env:USERPROFILE\.cargo\bin'" -ForegroundColor Gray
    Write-Host ""
    exit 0
}

Write-Host "[INFO] Rust not found. Installing..." -ForegroundColor Yellow
Write-Host ""

# Check if rustup-init exists
$rustupInit = "$env:USERPROFILE\Downloads\rustup-init.exe"
if (-not (Test-Path $rustupInit)) {
    Write-Host "[INFO] Downloading rustup-init.exe..." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Please download Rust installer:" -ForegroundColor Cyan
    Write-Host "  1. Visit: https://rustup.rs/" -ForegroundColor Gray
    Write-Host "  2. Download rustup-init.exe" -ForegroundColor Gray
    Write-Host "  3. Save to: $env:USERPROFILE\Downloads\" -ForegroundColor Gray
    Write-Host "  4. Run this script again" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Or run this command to download:" -ForegroundColor Cyan
    Write-Host "  Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile '$env:USERPROFILE\Downloads\rustup-init.exe'" -ForegroundColor Gray
    Write-Host ""
    
    $download = Read-Host "Download now? (Y/n)"
    if ($download -eq "" -or $download -eq "Y" -or $download -eq "y") {
        try {
            Write-Host "[INFO] Downloading..." -ForegroundColor Yellow
            Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile $rustupInit -UseBasicParsing
            Write-Host "[OK] Download completed" -ForegroundColor Green
        } catch {
            Write-Host "[ERROR] Download failed: $_" -ForegroundColor Red
            Write-Host "[INFO] Please download manually from: https://rustup.rs/" -ForegroundColor Yellow
            exit 1
        }
    } else {
        exit 0
    }
}

# Run rustup-init
Write-Host "[INFO] Running Rust installer..." -ForegroundColor Yellow
Write-Host "[INFO] Follow the prompts (press Enter for default options)" -ForegroundColor Cyan
Write-Host ""

Start-Process -FilePath $rustupInit -Wait -NoNewWindow

# Check if installation succeeded
Start-Sleep -Seconds 2

if (Test-Path $rustcPath) {
    Write-Host ""
    Write-Host "[SUCCESS] Rust installed successfully!" -ForegroundColor Green
    Write-Host ""
    
    # Add to PATH for current session
    $env:Path += ";$env:USERPROFILE\.cargo\bin"
    
    Write-Host "[INFO] Verifying installation..." -ForegroundColor Yellow
    & $rustcPath --version
    Write-Host ""
    
    Write-Host "[INFO] Adding Rust to PATH..." -ForegroundColor Yellow
    $currentPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ($currentPath -notlike "*$env:USERPROFILE\.cargo\bin*") {
        [Environment]::SetEnvironmentVariable('Path', $currentPath + ";$env:USERPROFILE\.cargo\bin", 'User')
        Write-Host "[OK] Added to PATH" -ForegroundColor Green
    } else {
        Write-Host "[INFO] Already in PATH" -ForegroundColor Gray
    }
    
    Write-Host ""
    Write-Host "[INFO] Please restart PowerShell for PATH changes to take effect" -ForegroundColor Yellow
    Write-Host "[INFO] Or run: `$env:Path += ';$env:USERPROFILE\.cargo\bin'" -ForegroundColor Gray
    Write-Host ""
} else {
    Write-Host ""
    Write-Host "[ERROR] Installation may have failed" -ForegroundColor Red
    Write-Host "[INFO] Please check the installer output above" -ForegroundColor Yellow
    Write-Host "[INFO] Or install manually from: https://rustup.rs/" -ForegroundColor Yellow
    Write-Host ""
}

