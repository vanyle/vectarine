#!/usr/bin/env pwsh

# This script installs/updates Vectarine for Windows x86_64 using PowerShell
# It automates the process of downloading and unzipping the latest release
# If you want to install it manually, you can download the latest release from https://github.com/vanyle/vectarine
# Running this script is just slightly more convenient, you do you <3 !

# Script is supposed to be compatible with PowerShell 5.1 and above.
if ($IsMacOS) {
    Write-Output "Please run the install_mac.sh script to install Vectarine on macOS."
    exit 1
} elseif ($IsLinux) {
    Write-Output "Please run the install_linux.sh script to install Vectarine on Linux."
    exit 1
}

$start_path = $PWD

Write-Output "Installing Vectarine for Windows..."
cd $HOME

$download_url = (Invoke-WebRequest "https://api.github.com/repos/vanyle/vectarine/releases/latest" | select -ExpandProperty "content" | ConvertFrom-Json | select -ExpandProperty "assets" | select -ExpandProperty "browser_download_url" |  Select-String -Pattern "windows").toString()

if (-not $download_url) {
    Write-Output "Failed to find the latest Windows release. Please check https://github.com/vanyle/vectarine/releases"
    cd $start_path
    exit 1
}

$temp_zip = "$env:TEMP\vectarine.zip"
$install_dir = "$env:APPDATA\vectarine"

Invoke-WebRequest $download_url -OutFile $temp_zip

# Remove existing installation if present
if (Test-Path $install_dir) {
    Write-Output "Updating existing installation..."
    Remove-Item -Recurse -Force $install_dir
}

# Extract
Write-Output "Extracting..."
Expand-Archive -Path $temp_zip -DestinationPath $install_dir -Force

# Clean up temp zip
Remove-Item -Force $temp_zip

# Verify the binary exists
$exe_path = "$install_dir\vecta.exe"
if (-not (Test-Path $exe_path)) {
    Write-Output "Warning: vecta.exe not found in the extracted files."
    cd $start_path
    exit 1
}


# Create a desktop shortcut if it doesn't already exist
$desktop = [Environment]::GetFolderPath("Desktop")
$shortcut_path = "$desktop\Vectarine.lnk"
if (-not (Test-Path $shortcut_path)) {
    Write-Output "Do you want to create a desktop shortcut for Vectarine? (Y/N)"
    $create_shortcut = Read-Host
    if ($create_shortcut -eq "Y" -or $create_shortcut -eq "y") {
        $shell = New-Object -ComObject WScript.Shell
        $shortcut = $shell.CreateShortcut($shortcut_path)
        $shortcut.TargetPath = $exe_path
        $shortcut.WorkingDirectory = $install_dir
        $shortcut.Description = "Vectarine"
        $shortcut.Save()
        Write-Output "A desktop shortcut for Vectarine has been created."
    }
}

Write-Output ""
Write-Output "You're all set!"
Write-Output "Vectarine is installed at: $install_dir"
Write-Output "If you want to run 'vecta' from the command line, add the following to your PATH:"
Write-Output "  $install_dir"
Write-Output ""
Write-Output "Note: To uninstall Vectarine, simply delete this folder."

cd $start_path

Write-Output ""
Write-Output "Do you want to open Vectarine now? (Y/N)"
$open_now = Read-Host
if ($open_now -eq "Y" -or $open_now -eq "y") {
    Start-Process $exe_path
}
