param([string]$NativeDirectory,[string]$RustDirectory,[string]$Output,[string[]]$Assets=@('Beech1','Wood','wood_floor'))
$ErrorActionPreference='Stop'
if(-not $NativeDirectory){$NativeDirectory=(Get-Content (Join-Path $PSScriptRoot 'private-cache\environment\native-current.txt') -Raw).Trim()}
$results=foreach($name in $Assets){
 $reference=@{}
 Import-Csv (Join-Path $NativeDirectory ($name+'.csv')) | ForEach-Object {$reference[$_.part+'|'+$_.index]=$_}
 $count=0;$maxError=0.;$maxAngle=0.
 foreach($a in Import-Csv (Join-Path $RustDirectory ($name+'.csv'))){
  $b=$reference[$a.part+'|'+$a.index];if(-not $b){throw ('Missing native vertex: '+$a.part+' '+$a.index)}
  $dx=[double]$a.x-[double]$b.x;$dy=[double]$a.y-[double]$b.y;$dz=[double]$a.z-[double]$b.z
  $maxError=[Math]::Max($maxError,[Math]::Sqrt($dx*$dx+$dy*$dy+$dz*$dz))
  $dot=[double]$a.nx*[double]$b.nx+[double]$a.ny*[double]$b.ny+[double]$a.nz*[double]$b.nz
  $maxAngle=[Math]::Max($maxAngle,[Math]::Acos([Math]::Max(-1.,[Math]::Min(1.,$dot)))*180./[Math]::PI);$count++
 }
 if($count -ne $reference.Count -or $count -eq 0 -or $maxError -gt 0.00005 -or $maxAngle -gt 0.1){throw ('Geometry comparison failed: '+$name)}
 [pscustomobject]@{asset=$name;verticesCompared=$count;maxPositionErrorMetres=$maxError;maxNormalDegrees=$maxAngle}
}
if($Output){if(Test-Path -LiteralPath $Output){throw 'Output exists; preserve it and choose a new path'};$results|ConvertTo-Json|Set-Content -LiteralPath $Output}
$results

