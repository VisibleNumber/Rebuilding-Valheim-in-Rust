$ErrorActionPreference = 'Stop'
$lab = $PSScriptRoot
$clone = Join-Path $lab 'test-game'
if (-not (Test-Path -LiteralPath (Join-Path $clone 'valheim_Data\Managed\ValheimLab.Diagnostic.dll'))) {throw 'Prepare and deploy the diagnostic first'}
$save = Join-Path $lab 'test-saves'
$log = Join-Path $lab ('evidence\manual-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '.log')
Start-Process -FilePath (Join-Path $clone 'valheim.exe') -WorkingDirectory $clone -ArgumentList @('-savedir', ('"' + $save + '"'), '-logFile', ('"' + $log + '"'), '-lab-record') -WindowStyle Hidden
Write-Output 'Test copy launched. Create a NEW character and LOCAL world. Recording starts automatically; F8 stops/starts it. Logs are in valheim-lab\recordings.'
