$ErrorActionPreference = 'Stop'
$lab = $PSScriptRoot
if(Get-Process valheim -ErrorAction SilentlyContinue){throw 'A Valheim session is running; preserving it'}
$clone = Join-Path $lab 'test-game'
if(-not (Test-Path -LiteralPath (Join-Path $clone '.valheim-lab-copy'))){throw 'Private test-copy marker missing'}
$log=Join-Path $lab ('evidence\greyling-stage-001\native-reference-'+(Get-Date -Format 'yyyyMMdd-HHmmss-ffff')+'.log')
$p=Start-Process -FilePath (Join-Path $clone 'valheim.exe') -WorkingDirectory $clone -ArgumentList @('-savedir',('"'+(Join-Path $lab 'test-saves')+'"'),'-logFile',('"'+$log+'"'),'-screen-fullscreen','0','-screen-width','1024','-screen-height','768','-lab-greyling-assets') -WindowStyle Hidden -PassThru
$p.WaitForExit()
[pscustomobject]@{utc=[DateTime]::UtcNow;exitCode=$p.ExitCode;log=$log;purpose='Native Greyling geometry reference and explicit Unity Idle-pose sampling; no world opened'} | ConvertTo-Json | Set-Content -LiteralPath ($log+'.result.json')
Write-Output ('Native reference exit code: '+$p.ExitCode)
if($p.ExitCode -ne 0){throw 'Native reference failed; retained log/cache contains failure details'}
