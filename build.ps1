$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    cargo build --release --locked --ignore-rust-version
    if ($LASTEXITCODE -ne 0) { throw 'Build failed' }
    New-Item -ItemType Directory -Path "$PSScriptRoot/dist" -Force | Out-Null
    Copy-Item -LiteralPath "$PSScriptRoot/target/release/openlogi-lighting-hotkey.exe" -Destination "$PSScriptRoot/dist/" -Force
    if (-not (Test-Path -LiteralPath "$PSScriptRoot/dist/settings.toml")) {
        Copy-Item -LiteralPath "$PSScriptRoot/settings.toml" -Destination "$PSScriptRoot/dist/"
    }
} finally { Pop-Location }
