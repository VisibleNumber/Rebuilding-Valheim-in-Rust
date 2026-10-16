$ErrorActionPreference='Stop'
$lab=$PSScriptRoot
if(Get-Process valheim -ErrorAction SilentlyContinue){throw 'Preserving a running Valheim session. Run this when it has finished.'}
$clone=Join-Path $lab 'test-game'
if(-not(Test-Path (Join-Path $clone '.valheim-lab-copy'))){throw 'Private-copy marker missing'}
$log=Join-Path $lab ('evidence\polish-001\native-'+(Get-Date -Format 'yyyyMMdd-HHmmss-ffff')+'.log')
$p=Start-Process (Join-Path $clone 'valheim.exe') -WorkingDirectory $clone -ArgumentList @('-savedir',('"'+(Join-Path $lab 'test-saves')+'"'),'-logFile',('"'+$log+'"'),'-screen-fullscreen','0','-screen-width','1024','-screen-height','768','-lab-environment') -WindowStyle Hidden -PassThru
$p.WaitForExit()
[pscustomobject]@{utc=[DateTime]::UtcNow;exitCode=$p.ExitCode;log=$log;purpose='Original LOD0 tree, wood and floor geometry/material/capture references in private copy'}|ConvertTo-Json|Set-Content ($log+'.result.json')
if($p.ExitCode -ne 0){throw 'Native reference failed. Inspect retained log and cache.'}
Get-Content (Join-Path $lab 'private-cache\environment\native-current.txt')
