param([string]$Compiler)
$ErrorActionPreference = 'Stop'
& "$PSScriptRoot/build.ps1"
if (-not $Compiler) {
    $command = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    if ($command) { $Compiler = $command.Source }
    else {
        $candidates = @(
            "${env:ProgramFiles(x86)}/Inno Setup 6/ISCC.exe",
            "$env:LOCALAPPDATA/Programs/Inno Setup 6/ISCC.exe",
            "$PSScriptRoot/tools/inno/ISCC.exe"
        )
        $Compiler = $candidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
    }
}
if (-not $Compiler) { throw 'Install Inno Setup 6, or pass -Compiler with the path to ISCC.exe.' }
& $Compiler "$PSScriptRoot/packaging/setup.iss"
if ($LASTEXITCODE -ne 0) { throw 'Installer compilation failed' }
