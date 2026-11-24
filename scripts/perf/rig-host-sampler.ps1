# Bench rig (t62-n12): Windows host process sampler for the quiet-host witness. Appends one JSON line per interval:
#   {"tMs":<unix ms, host clock>,"processes":{"<pid>":{"name":"<name>","cpuSeconds":<total processor seconds>}}}
# Processes whose CPU time cannot be read (protected processes) are left out. Stops when -StopFile exists or after
# -MaxSeconds. Its own PID is written as the first line ({"samplerPid":<pid>}) so the analyser can exclude it.
param(
 [Parameter(Mandatory)][string]$Out,
 [Parameter(Mandatory)][string]$StopFile,
 [ValidateRange(500,60000)][int]$IntervalMs=2000,
 [ValidateRange(1,86400)][int]$MaxSeconds=14400
)
$ErrorActionPreference='Stop'
$utf8=[Text.UTF8Encoding]::new($false)
[IO.File]::AppendAllText($Out,(@{samplerPid=$PID}|ConvertTo-Json -Compress)+"`n",$utf8)
$deadline=[DateTime]::UtcNow.AddSeconds($MaxSeconds)
while(-not (Test-Path -LiteralPath $StopFile) -and [DateTime]::UtcNow -lt $deadline){
 $due=[DateTime]::UtcNow.AddMilliseconds($IntervalMs)
 $processes=[ordered]@{}
 foreach($p in Get-Process){
  $cpu=$null;try{$cpu=$p.TotalProcessorTime.TotalSeconds}catch{$cpu=$null}
  if($null -ne $cpu){$processes["$($p.Id)"]=@{name=$p.ProcessName;cpuSeconds=[math]::Round($cpu,3)}}
 }
 $line=@{tMs=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds();processes=$processes}|ConvertTo-Json -Compress -Depth 4
 [IO.File]::AppendAllText($Out,$line+"`n",$utf8)
 $wait=($due-[DateTime]::UtcNow).TotalMilliseconds
 if($wait -gt 0){Start-Sleep -Milliseconds ([int]$wait)}
}
