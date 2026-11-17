param([switch]$Startup)
$ErrorActionPreference = 'Stop'
$destination = Join-Path $env:LOCALAPPDATA 'OpenLogiLightingHotkey'
New-Item -ItemType Directory -Path $destination -Force | Out-Null
Copy-Item -LiteralPath "$PSScriptRoot/dist/openlogi-lighting-hotkey.exe" -Destination $destination -Force
if (-not (Test-Path -LiteralPath "$destination/settings.toml")) {
    Copy-Item -LiteralPath "$PSScriptRoot/dist/settings.toml" -Destination $destination
}
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut((Join-Path ([Environment]::GetFolderPath('Desktop')) 'OpenLogi Lighting Hotkey.lnk'))
$shortcut.TargetPath = "$destination/openlogi-lighting-hotkey.exe"
$shortcut.WorkingDirectory = $destination
$shortcut.Save()
if ($Startup) {
    Copy-Item -LiteralPath $shortcut.FullName -Destination ([Environment]::GetFolderPath('Startup')) -Force
}
Write-Output "Installed to $destination. Launch using the desktop shortcut."
