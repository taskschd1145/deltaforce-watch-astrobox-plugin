# Run the pinned Rust toolchain without changing the user's PATH or global settings.
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$localCargo = Join-Path $projectRoot '.tools\cargo\bin\cargo.exe'
if (Test-Path -LiteralPath $localCargo) {
    $env:CARGO_HOME = Join-Path $projectRoot '.tools\cargo'
    $env:RUSTUP_HOME = Join-Path $projectRoot '.tools\rustup'
    $env:PATH = (Join-Path $projectRoot '.tools\cargo\bin') + ';' + $env:PATH
    & $localCargo @args
} else {
    & cargo @args
}
exit $LASTEXITCODE
