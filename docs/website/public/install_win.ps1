#!/usr/bin/env pwsh

$start_path = $PWD

Write-Output "Installing Vectarine for Windows..."
cd $HOME

$download_url = (Invoke-WebRequest "https://api.github.com/repos/vanyle/vectarine/releases/latest" | select -ExpandProperty "content" | ConvertFrom-Json | select -ExpandProperty "assets" | select -ExpandProperty "browser_download_url" |  Select-String -Pattern "windows").toString()

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

# Create a desktop shortcut (or override existing one)
$desktop = [Environment]::GetFolderPath("Desktop")
$shortcut_path = "$desktop\Vectarine.lnk"

$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($shortcut_path)
$shortcut.TargetPath = $exe_path
$shortcut.WorkingDirectory = $install_dir
$shortcut.Description = "Vectarine"
$shortcut.Save()

Write-Output ""
Write-Output "You're all set! A shortcut has been placed on your Desktop."
Write-Output "Vectarine is installed at: $install_dir"
Write-Output "If you want to run 'vecta' from the command line, add the following to your PATH:"
Write-Output "  $install_dir"

cd $start_path
