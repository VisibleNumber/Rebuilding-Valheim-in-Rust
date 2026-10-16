param([switch]$Reload, [switch]$Render)
$ErrorActionPreference = 'Stop'
$lab = $PSScriptRoot
$clone = Join-Path $lab 'test-game'
$exe = Join-Path $clone 'valheim.exe'
if (-not (Test-Path -LiteralPath (Join-Path $clone '.valheim-lab-copy'))) {throw 'Private test copy missing'}
if (Get-Process valheim -ErrorAction SilentlyContinue) {throw 'A Valheim session is already running; preserving it'}
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss-ffff'
$log = Join-Path $lab ('evidence\runtime-' + $(if($Reload){'reload'}else{'scenario'}) + '-' + $stamp + '.log')
$save = Join-Path $lab 'test-saves'
$runArgs = @('-savedir', ('"' + $save + '"'), '-logFile', ('"' + $log + '"'), '-lab-selftest')
if (-not $Render) {$runArgs += @('-batchmode','-nographics')}
else {$runArgs += @('-screen-fullscreen','0','-screen-width','960','-screen-height','540','-lab-capture')}
if ($Reload) {$runArgs += '-lab-reload'}
$p = Start-Process -FilePath $exe -WorkingDirectory $clone -ArgumentList $runArgs -WindowStyle Hidden -PassThru
$p.Id | Set-Content -LiteralPath (Join-Path $lab 'evidence\active-test-process.txt')
$p.WaitForExit()
[pscustomobject]@{utc=[DateTime]::UtcNow;pid=$p.Id;exitCode=$p.ExitCode;log=$log;reload=[bool]$Reload;render=[bool]$Render} | ConvertTo-Json | Set-Content -LiteralPath ($log + '.result.json')
Write-Output ('Test game exit code: ' + $p.ExitCode + '; log: ' + $log)
if ($p.ExitCode -ne 0) {throw 'Native gameplay test failed; inspect its log and selftest result'}
