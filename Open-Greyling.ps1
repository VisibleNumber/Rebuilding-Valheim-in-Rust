param([switch]$CheckOnly)
$ErrorActionPreference='Stop'
$lab=$PSScriptRoot
$game=[IO.Path]::GetFullPath((Join-Path $lab '..\common\Valheim'))
$expected=@{
 'valheim.exe'='3ECC2EEA5CE2ACFCAA6868AB7D8252B9A07910EA440405E62817890F4EF6BD50'
 'valheim_Data\Managed\assembly_valheim.dll'='94998885B103D6E5F5480123EAEE0A093E82AF2DDEEEE37536C15BF555F24F3A'
 'valheim_Data\StreamingAssets\SoftRef\Bundles\c4210710'='2FDCF6840C7A5EA64A3F19B63BB3047D5884C202BA67B7AFC997491F635A1B55'
 'valheim_Data\StreamingAssets\SoftRef\manifest_extended'='75D44C508602A0AF85E7881BB8F99D3EA9E77F7DD39FF305070FDE7A2FEB1B64'
}
function Get-InstalledHash([string]$Path) {
 $stream=[IO.File]::OpenRead($Path)
 $sha=[Security.Cryptography.SHA256]::Create()
 try {return [BitConverter]::ToString($sha.ComputeHash($stream)).Replace('-','')}finally{$sha.Dispose();$stream.Dispose()}
}
Write-Output 'Checking the installed Valheim asset build...'
foreach($entry in $expected.GetEnumerator()){
 $path=Join-Path $game $entry.Key
 if(-not (Test-Path -LiteralPath $path)){throw ('Required installed file missing: '+$entry.Key)}
 if((Get-InstalledHash $path) -ne $entry.Value){throw ('Valheim has changed. Reinspect this build before using the saved asset profile: '+$entry.Key)}
}
Push-Location $lab
try {
 if(-not (Test-Path -LiteralPath 'greyling-viewer\bin\greyling-viewer.exe')){& .\Build-Greyling.ps1}
 if(-not (Test-Path -LiteralPath 'private-cache\greyling\serialized-c421-001\summary.txt')){
  if(Test-Path -LiteralPath 'private-cache\greyling\serialized-c421-001'){throw 'An incomplete metadata cache exists; preserve and inspect it before regeneration'}
  & .\asset-probe\bin\asset-probe.exe --inspect-bundle (Join-Path $game 'valheim_Data\StreamingAssets\SoftRef\Bundles\c4210710') private-cache\greyling\serialized-c421-001
  if($LASTEXITCODE -ne 0){throw 'Installed bundle metadata inspection failed'}
 }
 if($CheckOnly){& .\greyling-viewer\bin\greyling-viewer.exe --check}else{& .\greyling-viewer\bin\greyling-viewer.exe}
 if($LASTEXITCODE -ne 0){throw 'Greyling viewer failed; see the console output'}
}finally{Pop-Location}

