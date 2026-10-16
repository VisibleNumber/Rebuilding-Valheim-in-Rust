param([switch]$CheckOnly,[string]$Capture)
$ErrorActionPreference='Stop'
Push-Location $PSScriptRoot
try {
 & .\Open-Greyling.ps1 -CheckOnly
 if(-not (Test-Path playable-world\ValheimSandbox.exe)){& .\Build-Playable.ps1}
 Write-Host 'Click to play | Mouse look | WASD move | Shift run | Q jump | Space strike'
 Write-Host 'E gather | B preview/place | V cancel | Tab pause/release mouse | H help | F5 save | F9 load'
 if($CheckOnly){return}
 if($Capture){& .\playable-world\ValheimSandbox.exe --capture $Capture}else{& .\playable-world\ValheimSandbox.exe}
 if($LASTEXITCODE -ne 0){throw 'Sandbox exited with an error'}
}finally{Pop-Location}


