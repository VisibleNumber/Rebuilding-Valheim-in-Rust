$ErrorActionPreference = 'Stop'
$lab = $PSScriptRoot
$game = [IO.Path]::GetFullPath((Join-Path $lab '..\common\Valheim'))
$clone = Join-Path $lab 'test-game'
if (-not (Test-Path -LiteralPath (Join-Path $game 'valheim.exe'))) { throw 'Installed Valheim not found' }
if (Test-Path -LiteralPath $clone) { throw 'Test copy already exists; preserving it. Use the patcher to redeploy a backed-up diagnostic.' }
New-Item -ItemType Directory -Path $clone | Out-Null
Get-ChildItem -LiteralPath $game -Force | Copy-Item -Destination $clone -Recurse
Set-Content -LiteralPath (Join-Path $clone '.valheim-lab-copy') -Value 'Private test copy. Never distribute game files.'
New-Item -ItemType Directory -Path (Join-Path $lab 'test-saves') -Force | Out-Null
Write-Output 'Separate game copy prepared. Original game and saves remain intact.'
