$ErrorActionPreference='Stop'
Push-Location $PSScriptRoot
try {
 New-Item -ItemType Directory -Force greyling-viewer\bin,asset-probe\bin | Out-Null
 & rustc --edition 2021 -O -Awarnings asset-probe\main.rs -o asset-probe\bin\asset-probe.exe
 if($LASTEXITCODE -ne 0){throw 'Asset reader build failed'}
 & rustc --edition 2021 -O -Awarnings greyling-viewer\main.rs -o greyling-viewer\bin\greyling-viewer.exe
 if($LASTEXITCODE -ne 0){throw 'Greyling viewer build failed'}
 & rustc --edition 2021 -O -Awarnings greyling-viewer\inspect.rs -o greyling-viewer\bin\inspect-greyling.exe
 if($LASTEXITCODE -ne 0){throw 'Reference graph tool build failed'}
 & rustc --edition 2021 --test -Awarnings asset-probe\main.rs -o asset-probe\bin\asset-probe-tests.exe
 if($LASTEXITCODE -ne 0){throw 'Decoder test build failed'}
 & .\asset-probe\bin\asset-probe-tests.exe
 if($LASTEXITCODE -ne 0){throw 'Decoder tests failed'}
} finally {Pop-Location}
