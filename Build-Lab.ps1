$ErrorActionPreference = 'Stop'
$env:DOTNET_CLI_HOME = Join-Path $PSScriptRoot '.dotnet'
$env:DOTNET_CLI_TELEMETRY_OPTOUT = '1'
foreach ($project in @('inspector','diagnostic','patcher')) {
    & dotnet build (Join-Path $PSScriptRoot $project) -c Release --ignore-failed-sources
    if ($LASTEXITCODE -ne 0) { throw "Build failed: $project" }
}
New-Item -ItemType Directory -Force (Join-Path $PSScriptRoot 'asset-probe\bin') | Out-Null
& rustc --edition 2021 -O (Join-Path $PSScriptRoot 'asset-probe\main.rs') -o (Join-Path $PSScriptRoot 'asset-probe\bin\asset-probe.exe')
if ($LASTEXITCODE -ne 0) {throw 'Rust asset probe build failed'}
& rustc --edition 2021 --test (Join-Path $PSScriptRoot 'asset-probe\main.rs') -o (Join-Path $PSScriptRoot 'asset-probe\bin\asset-probe-tests.exe')
if ($LASTEXITCODE -ne 0) {throw 'Rust test build failed'}
& (Join-Path $PSScriptRoot 'asset-probe\bin\asset-probe-tests.exe')
if ($LASTEXITCODE -ne 0) {throw 'Rust asset probe tests failed'}
& (Join-Path $PSScriptRoot 'Build-Greyling.ps1')
