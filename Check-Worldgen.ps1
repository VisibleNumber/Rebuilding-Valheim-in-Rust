param([string]$ReferenceDirectory)
$ErrorActionPreference='Stop'
Push-Location $PSScriptRoot
try {
 if(-not $ReferenceDirectory){$ReferenceDirectory=(Get-Content private-cache\worldgen\native-current.txt -Raw).Trim()}
 & rustc --edition 2021 -O playable-world\worldgen_probe.rs -o evidence\worldgen-002\worldgen-probe.exe
 if($LASTEXITCODE -ne 0){throw 'Worldgen probe compilation failed'}
 & .\evidence\worldgen-002\worldgen-probe.exe $ReferenceDirectory
 if($LASTEXITCODE -ne 0){throw 'Native checkpoint comparison failed'}
}finally{Pop-Location}
