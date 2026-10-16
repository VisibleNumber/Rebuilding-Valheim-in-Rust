$ErrorActionPreference='Stop'
Push-Location $PSScriptRoot
try {
 & rustc --edition 2021 -O -Awarnings playable-world\main.rs -o playable-world\ValheimSandbox.exe
 if($LASTEXITCODE -ne 0){throw 'Playable sandbox build failed'}
} finally {Pop-Location}
