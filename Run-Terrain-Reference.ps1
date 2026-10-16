param([switch]$InspectOnly)
$ErrorActionPreference='Stop'
$lab=$PSScriptRoot
if(Get-Process valheim -ErrorAction SilentlyContinue){throw 'Preserving a running Valheim session. Run this when it has finished.'}
$clone=Join-Path $lab 'test-game'
if(-not(Test-Path (Join-Path $clone '.valheim-lab-copy'))){throw 'Private test-copy marker missing'}
$stage=if($InspectOnly){'terrain-patch'}else{'terrain-patch'}
$log=Join-Path $lab ('evidence\'+$stage+'\native-'+(Get-Date -Format 'yyyyMMdd-HHmmss-ffff')+'.log')
New-Item -ItemType Directory -Force (Split-Path $log) | Out-Null
$mode=if($InspectOnly){'-lab-terrain-inspect'}else{'-lab-terrain'}
$p=Start-Process -FilePath (Join-Path $clone 'valheim.exe') -WorkingDirectory $clone -ArgumentList @('-savedir',('"'+(Join-Path $lab 'test-saves')+'"'),'-logFile',('"'+$log+'"'),'-screen-fullscreen','0','-screen-width','1024','-screen-height','768',$mode) -WindowStyle Hidden -PassThru
$p.WaitForExit()
[pscustomobject]@{utc=[DateTime]::UtcNow;exitCode=$p.ExitCode;log=$log;purpose='Native terrain generator and installed collision mesh reference; dedicated private cache'} | ConvertTo-Json | Set-Content ($log+'.result.json')
if($p.ExitCode -ne 0){throw 'Native reference failed. Retained log/cache contain the failure.'}
if($InspectOnly){Get-Content (Join-Path $lab 'private-cache\terrain\inspection-current.txt')}else{
 $patch=Get-Content (Join-Path $lab 'private-cache\terrain\patch-current.txt')
 $evidence=Join-Path $lab 'evidence\meadows-001'
 New-Item -ItemType Directory -Force $evidence | Out-Null
 foreach($name in @('patch.tsv','identity.txt','tile-bounds.tsv','tile-biomes.tsv','seams.tsv','native-camera.txt','native-overview.png')){
  $source=Join-Path $patch $name
  if(Test-Path -LiteralPath $source){Copy-Item -LiteralPath $source -Destination (Join-Path $evidence ('terrain-'+$name)) -Force}
 }
 $envDir=Join-Path $patch 'environment'
 if(Test-Path -LiteralPath $envDir){Get-ChildItem -LiteralPath $envDir -File | ForEach-Object {Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $evidence ('environment-'+$_.Name)) -Force}}
 Write-Output $patch
}
