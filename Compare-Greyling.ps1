param([string]$NativeDirectory, [string]$OutputDirectory)
$ErrorActionPreference='Stop'
$lab=$PSScriptRoot
if(-not $NativeDirectory){$NativeDirectory=(Get-Content -LiteralPath (Join-Path $lab 'private-cache\greyling\native-current.txt') -Raw).Trim()}
if(-not $OutputDirectory){$OutputDirectory=Join-Path $lab ('evidence\greyling-stage-001\comparison-'+(Get-Date -Format 'yyyyMMdd-HHmmss-ffff'))}
if(Test-Path -LiteralPath $OutputDirectory){throw 'Comparison output already exists; preserving it'}
New-Item -ItemType Directory -Path $OutputDirectory | Out-Null
Push-Location $lab
try {
 $summaries=foreach($frame in @('rest','0','30','60')){
  $rust=Join-Path $OutputDirectory ('rust-vertices-'+$frame+'.csv')
  $runArgs=@('--check','--dump',$rust)
  if($frame -ne 'rest'){$runArgs+=@('--pose',$frame)}
  & (Join-Path $lab 'greyling-viewer\bin\greyling-viewer.exe') @runArgs > (Join-Path $OutputDirectory ('reader-'+$frame+'.log'))
  if($LASTEXITCODE -ne 0){throw 'Rust vertex export failed'}
  $native=Join-Path $NativeDirectory $(if($frame -eq 'rest'){'native-rest-vertices.csv'}else{'native-idle-vertices-'+$frame+'.csv'})
  $nativeRows=Import-Csv -LiteralPath $native
  $rustRows=Import-Csv -LiteralPath $rust
  $lookup=@{};foreach($r in $rustRows){$key=$r.part+'/'+$r.index;if($lookup.ContainsKey($key)){throw 'Duplicate Rust vertex key'};$lookup[$key]=$r}
  if($nativeRows.Count -ne $rustRows.Count){throw 'Native/Rust vertex counts differ'}
  $errors=foreach($n in $nativeRows){
   $r=$lookup[$n.part+'/'+$n.index];if(-not $r){throw 'Missing corresponding vertex'}
   $squared=0.;$dot=0.;foreach($axis in @('x','y','z')){
    $nv=[double]::Parse($n.$axis,[Globalization.CultureInfo]::InvariantCulture);$rv=[double]::Parse($r.$axis,[Globalization.CultureInfo]::InvariantCulture);$squared+=($nv-$rv)*($nv-$rv)
    $normal='n'+$axis;$dot+=[double]::Parse($n.$normal,[Globalization.CultureInfo]::InvariantCulture)*[double]::Parse($r.$normal,[Globalization.CultureInfo]::InvariantCulture)
   }
   [pscustomobject]@{part=$n.part;index=$n.index;positionErrorMetres=[Math]::Sqrt($squared);normalAngleDegrees=[Math]::Acos([Math]::Clamp($dot,-1.,1.))*180/[Math]::PI}
  }
  $errors|Export-Csv -LiteralPath (Join-Path $OutputDirectory ('errors-'+$frame+'.csv')) -NoTypeInformation
  [pscustomobject]@{pose=$frame;vertices=$errors.Count;maxPositionErrorMetres=($errors|Measure-Object positionErrorMetres -Maximum).Maximum;meanPositionErrorMetres=($errors|Measure-Object positionErrorMetres -Average).Average;maxNormalAngleDegrees=($errors|Measure-Object normalAngleDegrees -Maximum).Maximum;native=$native;rust=$rust}
 }
 $summaries|ConvertTo-Json -Depth 4|Set-Content -LiteralPath (Join-Path $OutputDirectory 'summary.json')
 $summaries|Format-Table pose,vertices,maxPositionErrorMetres,meanPositionErrorMetres,maxNormalAngleDegrees
 if($summaries|Where-Object {$_.maxPositionErrorMetres -gt 0.00001 -or $_.maxNormalAngleDegrees -gt 0.1}){throw 'Geometry comparison exceeded 10 micrometres or 0.1 degrees; do not claim parity'}
 Write-Output ('Geometry comparisons passed. Evidence: '+$OutputDirectory)
} finally {Pop-Location}
