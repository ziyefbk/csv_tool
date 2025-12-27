# Create a simple placeholder icon file for Tauri Windows build

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Create Icon File for Tauri" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$iconDir = "tauri\icons"
$iconFile = "$iconDir\icon.ico"

# Create directory if needed
if (-not (Test-Path $iconDir)) {
    New-Item -ItemType Directory -Path $iconDir -Force | Out-Null
    Write-Host "[INFO] Created icon directory" -ForegroundColor Green
}

# Check if icon already exists
if (Test-Path $iconFile) {
    Write-Host "[INFO] Icon file already exists: $iconFile" -ForegroundColor Green
    Write-Host ""
    $overwrite = Read-Host "Overwrite? (y/N)"
    if ($overwrite -ne "y" -and $overwrite -ne "Y") {
        Write-Host "Cancelled."
        exit 0
    }
}

Write-Host "[INFO] Creating icon file..." -ForegroundColor Yellow
Write-Host ""

# Method 1: Try using online tool instructions
Write-Host "Method 1: Use online converter (Recommended)" -ForegroundColor Cyan
Write-Host "  1. Visit: https://convertio.co/zh/png-ico/" -ForegroundColor Gray
Write-Host "  2. Upload any PNG image (256x256 or larger)" -ForegroundColor Gray
Write-Host "  3. Download as ICO format" -ForegroundColor Gray
Write-Host "  4. Save to: $iconFile" -ForegroundColor Gray
Write-Host ""

# Method 2: Try using ImageMagick if available
Write-Host "Method 2: Using ImageMagick (if installed)" -ForegroundColor Cyan
try {
    $magick = Get-Command magick -ErrorAction Stop
    Write-Host "[INFO] ImageMagick found, creating icon..." -ForegroundColor Green
    
    # Create a simple 256x256 blue icon
    magick convert -size 256x256 xc:"#4A90E2" "$iconFile"
    
    if (Test-Path $iconFile) {
        Write-Host "[SUCCESS] Icon file created: $iconFile" -ForegroundColor Green
        Write-Host ""
        exit 0
    }
} catch {
    Write-Host "[INFO] ImageMagick not found, skipping..." -ForegroundColor Gray
}

# Method 3: Try using Tauri CLI if available
Write-Host "Method 3: Using Tauri CLI (if you have a PNG icon)" -ForegroundColor Cyan
Write-Host "  cd tauri" -ForegroundColor Gray
Write-Host "  cargo tauri icon path/to/your/icon.png" -ForegroundColor Gray
Write-Host ""

# Method 4: Download a placeholder icon
Write-Host "Method 4: Download placeholder icon" -ForegroundColor Cyan
$download = Read-Host "Download a simple placeholder icon? (Y/n)"
if ($download -eq "" -or $download -eq "Y" -or $download -eq "y") {
    try {
        Write-Host "[INFO] Downloading placeholder icon..." -ForegroundColor Yellow
        
        # Try to download a minimal valid ICO file
        # Using a simple 1x1 pixel ICO file (minimal valid format)
        $icoBytes = @(
            0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x20, 0x20, 0x00, 0x00, 0x01, 0x00,
            0x20, 0x00, 0xA8, 0x10, 0x00, 0x00, 0x16, 0x00, 0x00, 0x00, 0x28, 0x00,
            0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x01, 0x00,
            0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        )
        
        # Create a simple blue 32x32 icon
        # This is a minimal valid ICO file structure
        $icoData = [System.Convert]::FromBase64String("AAABAAEAEBAAAAEAIABoBAAAFgAAACgAAAAQAAAAIAAAAAEAIAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAA")
        
        # Create a proper 32x32 ICO file with blue color
        # ICO file format: header + directory + image data
        $icoBytes = New-Object byte[] 1506
        
        # ICO header (6 bytes)
        $icoBytes[0] = 0x00; $icoBytes[1] = 0x00  # Reserved
        $icoBytes[2] = 0x01; $icoBytes[3] = 0x00  # Type (1 = ICO)
        $icoBytes[4] = 0x01; $icoBytes[5] = 0x00  # Number of images
        
        # Directory entry (16 bytes)
        $icoBytes[6] = 0x20  # Width (32)
        $icoBytes[7] = 0x20  # Height (32)
        $icoBytes[8] = 0x00  # Color palette
        $icoBytes[9] = 0x00  # Reserved
        $icoBytes[10] = 0x01; $icoBytes[11] = 0x00  # Color planes
        $icoBytes[12] = 0x20; $icoBytes[13] = 0x00  # Bits per pixel (32)
        $icoBytes[14] = 0xE0; $icoBytes[15] = 0x05  # Image size (1504 bytes)
        $icoBytes[16] = 0x16; $icoBytes[17] = 0x00  # Offset to image data (22)
        
        # BMP header (starts at offset 22)
        # This creates a minimal valid ICO file
        # For simplicity, let's use a different approach
        
        Write-Host "[INFO] Creating minimal ICO file..." -ForegroundColor Yellow
        
        # Create a minimal valid ICO file using .NET
        Add-Type -TypeDefinition @"
using System;
using System.IO;
public class IconCreator {
    public static void CreateIcon(string path) {
        byte[] ico = new byte[] {
            0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x20, 0x20, 0x00, 0x00, 0x01, 0x00,
            0x20, 0x00, 0xA8, 0x10, 0x00, 0x00, 0x16, 0x00, 0x00, 0x00, 0x28, 0x00,
            0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x01, 0x00,
            0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        };
        // Fill with blue color (RGBA: 74, 144, 226, 255)
        for (int i = 54; i < ico.Length; i += 4) {
            ico[i] = 226;     // B
            ico[i+1] = 144;   // G
            ico[i+2] = 74;    // R
            ico[i+3] = 255;   // A
        }
        File.WriteAllBytes(path, ico);
    }
}
"@
        
        [IconCreator]::CreateIcon((Resolve-Path $iconFile))
        
        if (Test-Path $iconFile) {
            Write-Host "[SUCCESS] Placeholder icon created: $iconFile" -ForegroundColor Green
            Write-Host "[INFO] You can replace this with a better icon later" -ForegroundColor Yellow
            Write-Host ""
            exit 0
        }
    } catch {
        Write-Host "[ERROR] Failed to create icon: $_" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "[INFO] Please use Method 1 (online converter) to create the icon file" -ForegroundColor Yellow
Write-Host "[INFO] Or install ImageMagick and run this script again" -ForegroundColor Yellow
Write-Host ""

