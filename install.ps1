$ErrorActionPreference = "Stop"

$Repo = "YOUR_GITHUB_USERNAME/dead-simple-cli-timer"
$Bin = "dstimer"
$InstallDir = "$env:LOCALAPPDATA\Programs\$Bin"
$Asset = "$Bin-windows-x86_64.exe"

# Resolve latest release tag
$Release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
$Tag = $Release.tag_name

if (-not $Tag) {
    Write-Error "Could not determine latest release tag."
    exit 1
}

$Url = "https://github.com/$Repo/releases/download/$Tag/$Asset"

Write-Host "Installing $Bin $Tag (windows/x86_64)..."

# Download
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$Dest = "$InstallDir\$Bin.exe"
Invoke-WebRequest -Uri $Url -OutFile $Dest

Write-Host "Installed to $Dest"

# Add to PATH for current user if not already present
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$InstallDir", "User")
    Write-Host ""
    Write-Host "  Added $InstallDir to your PATH."
    Write-Host "  Restart your terminal for the change to take effect."
}
