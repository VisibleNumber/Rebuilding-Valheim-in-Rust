param([string]$Output)
$ErrorActionPreference='Stop'
Push-Location $PSScriptRoot
try {
 if(-not $Output){$Output=Join-Path $PSScriptRoot ('evidence\meadows-001\verify-'+(Get-Date -Format 'yyyyMMdd-HHmmss'))}
 if(Test-Path -LiteralPath $Output){throw 'Choose a fresh verification output directory'}
 New-Item -ItemType Directory -Path $Output | Out-Null
 & .\Open-Playable.ps1 -CheckOnly *> (Join-Path $Output 'installed-build.log')
 & .\Build-Playable.ps1
 $exe=Join-Path $PSScriptRoot 'playable-world\ValheimSandbox.exe'
 $patch=(Get-Content private-cache\terrain\patch-current.txt -Raw).Trim()
 function Run-Check([string]$Name,[string[]]$Arguments) {
  & $exe @Arguments *> (Join-Path $Output ($Name+'.log'))
  if($LASTEXITCODE -ne 0){throw ('Check failed: '+$Name+'; retained log and outputs')}
 }
 Run-Check 'terrain' @('--verify-patch',$patch)
 $save=Join-Path $Output 'loop.save'
 Run-Check 'loop' @('--scenario',$save)
 Run-Check 'restart' @('--verify-save',$save)
 Run-Check 'area' @('--area-scenario',(Join-Path $Output 'area'))
 Run-Check 'window' @('--playtest',(Join-Path $Output 'window'))
 Run-Check 'rendered-restart' @('--load-save',(Join-Path $Output 'window\window-test.save'),'--capture',(Join-Path $Output 'reopened.bmp'))
 Run-Check 'traversal' @('--traverse',(Join-Path $Output 'traversal'))
 Run-Check 'assets' @('--asset-dump',(Join-Path $Output 'assets'))
 & .\Compare-Environment.ps1 -RustDirectory (Join-Path $Output 'assets') -Output (Join-Path $Output 'asset-comparison.json') -Assets @('Beech1','Wood','wood_floor','Rock_7_meadows','Bush01','Rock_3','Rock_4','Beech_small1','Beech_small2','Birch1') *> (Join-Path $Output 'asset-comparison.log')
 $desktop=Join-Path $Output 'desktop.bmp'
 $desktopCommand='""'+(Join-Path $PSScriptRoot 'Open Playable World.cmd')+'" -Capture "'+$desktop+'""'
 $desktopProcess=Start-Process -FilePath $env:ComSpec -ArgumentList ('/d /s /c '+$desktopCommand) -WorkingDirectory $PSScriptRoot -WindowStyle Hidden -Wait -PassThru -RedirectStandardOutput (Join-Path $Output 'desktop.log') -RedirectStandardError (Join-Path $Output 'desktop-error.log')
 if($desktopProcess.ExitCode -ne 0){throw 'CMD -> built-in Windows PowerShell launcher check failed'}
 if(-not(Test-Path -LiteralPath $desktop)){throw 'Desktop launcher did not produce its required capture'}
 Get-FileHash -Algorithm SHA256 -LiteralPath $exe | Export-Csv (Join-Path $Output 'binary-hash.csv') -NoTypeInformation
 Write-Output ('Completed automated checks: '+$Output)
} finally {Pop-Location}

