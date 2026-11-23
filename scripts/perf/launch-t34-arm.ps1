# Launches ONE t34 A/B run (one arm, one Docker app + generator pair) from scripts/perf/t34-campaign.json.
# Derived from the 2026-11-23 baseline launcher (.orchestration/reports/perf-investigation/t34-baseline-20261123/launch-t34-bench0.ps1):
# same limits and fixture settings; the only additions are the arm's environment (from `t34-modes.mjs env <arm>`) and the
# report folder/scratch location parameters. It does not build the image and does not wait for the campaign; the caller
# waits for the generator container to exit, then runs `node scripts/perf/t34-modes.mjs check <report> --arm <arm>`.
param(
 [Parameter(Mandatory)][ValidatePattern('^[a-z][a-z0-9]*$')][string]$Run,
 [Parameter(Mandatory)][ValidateSet('baseline','original','snapshot','journal','write-behind','all','all-original')][string]$Arm,
 [Parameter(Mandatory)][string]$Image,
 [Parameter(Mandatory)][string]$ReportRoot,
 [Parameter(Mandatory)][string]$PrivateRoot,
 [ValidateRange(1,60)][int]$Seconds=30,
 [string]$Endpoints='identity,query,headers,representative',
 [string]$Concurrency='8,32,64',
 [string]$AppMemory='4g',
 [switch]$RequestProfile
)
$ErrorActionPreference='Stop'
$docker='C:/Program Files/Docker/Docker/resources/bin/docker.exe'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$prefix="t34-ab-$Run"
$reportDir=Join-Path $ReportRoot $Run
$privateDir=Join-Path $PrivateRoot "private-$Run"
if(Test-Path -LiteralPath $reportDir){throw 'Existing evidence is retained; choose a new Run'}
$armLines=@(& node (Join-Path $repo 'scripts/perf/t34-modes.mjs') env $Arm)
if($LASTEXITCODE -ne 0){throw "Invalid arm $Arm"}
New-Item -ItemType Directory -Path $reportDir,$privateDir -Force | Out-Null
function D([string[]]$a){$r=& $docker @a;if($LASTEXITCODE -ne 0){throw "Docker operation failed: $($a[0])"};return $r}
$scratch="$prefix-scratch";$data="$prefix-data"
D @('volume','create','--label',"mw.perf.owner=$prefix",$scratch)|Out-Null
D @('volume','create','--label',"mw.perf.owner=$prefix",$data)|Out-Null
$setup="const fs=require('fs');for(const p of ['/scratch/control','/scratch/reports']){fs.mkdirSync(p,{recursive:true,mode:0o700});fs.chmodSync(p,0o700);}"
D @('run','--rm','--name',"$prefix-setup",'--label',"mw.perf.owner=$prefix",'--network','none','--mount',"type=volume,source=$scratch,target=/scratch",'--entrypoint','node',$Image,'-e',$setup)|Out-Null
$key=[Convert]::ToHexString([Security.Cryptography.RandomNumberGenerator]::GetBytes(32)).ToLowerInvariant()
$envFile=Join-Path $privateDir 'app.env'
$values=@(
 'MW_DB_ENCRYPTION=required',"MW_DB_KEY=$key",
 'MW_NATIVE_LOAD_PRINCIPALS=100','MW_NATIVE_LOAD_WORKLOAD=representative','MW_NATIVE_LOAD_MESSAGES=50',
 'MW_LOAD_FIXTURE_EXTERNAL_DRIVER=1','MW_LOAD_FIXTURE_EXTERNAL_ROOT=/scratch/control',
 'MW_LOAD_REPORT_DIR=/scratch/reports','MW_LOAD_FIXTURE_MODE=ramp','MW_LOAD_FIXTURE_LONG=1',
 "MW_LOAD_FIXTURE_SECONDS=$Seconds",'MW_LOAD_FIXTURE_WARMUP_SECONDS=20',
 'MW_LOAD_FIXTURE_DEADLINE_SECONDS=1800',"MW_LOAD_FIXTURE_CONCURRENCY=$Concurrency",
 "MW_LOAD_FIXTURE_ENDPOINTS=$Endpoints",'MW_LOAD_FIXTURE_WORKERS=4',
 'MW_NATIVE_LOAD_POLICY=loaded',"MW_FIXTURE_REQUEST_PROFILE=$(if($RequestProfile){'1'}else{'0'})",
 'MW_NATIVE_LOAD_INDEX_TIMING=0','MW_NATIVE_LOAD_INDEX_WORKERS=3','MW_NATIVE_LOAD_INDEX_MODE=sealed',
 'MW_NATIVE_LOAD_SECURITY_PROFILE=enforced'
)+$armLines
[IO.File]::WriteAllLines($envFile,$values,[Text.UTF8Encoding]::new($false))
& icacls $privateDir /inheritance:r /grant:r "${env:USERNAME}:(OI)(CI)F" | Out-Null
if($LASTEXITCODE -ne 0){throw 'Could not restrict fixture key directory'}
$app="$prefix-app"
$appId=D @('run','-d','--name',$app,'--label',"mw.perf.owner=$prefix",'--network','none',
 '--cpus','4','--cpuset-cpus','0-3','--memory',$AppMemory,'--memory-swap',$AppMemory,'--ulimit','nofile=65536:65536',
 '--mount',"type=volume,source=$scratch,target=/scratch",'--mount',"type=volume,source=$data,target=/tmp",
 '--env-file',$envFile,$Image)
$generator="$prefix-generator"
$generatorId=D @('run','-d','--name',$generator,'--label',"mw.perf.owner=$prefix",'--network',"container:$app",
 '--cpus','2','--cpuset-cpus','8-9','--memory','1g','--memory-swap','1g','--ulimit','nofile=65536:65536',
 '--mount',"type=volume,source=$scratch,target=/scratch",'--entrypoint','node',$Image,
 '/src/scripts/perf/http-external-driver.mjs','--root=/scratch/control','--campaigns=1','--wait-seconds=1800','--workers=2')
$metadata=[ordered]@{owner=$prefix;run=$Run;arm=$Arm;armEnvironment=$armLines;app=$app;appId=$appId;generator=$generator;generatorId=$generatorId;scratchVolume=$scratch;dataVolume=$data;
 securityProfile='enforced';policy='loaded';workload='representative';messages=50;principals=100;database='sqlite';wholeDatabaseEncryption='required';
 requestProfile=[bool]$RequestProfile;instrumentedRun=[bool]$RequestProfile;imageId=(D @('image','inspect','--format','{{.Id}}',$Image));
 seconds=$Seconds;warmupSeconds=20;concurrency=$Concurrency;endpoints=$Endpoints;
 appResources=@{cpu=4;memory=$AppMemory;cpuset='0-3';swapAllowed=$false};generatorResources=@{cpu=2;memory='1g';cpuset='8-9';workers=2};
 privateEnvironment='outside the repository (not retained)';createdAtUtc=[DateTime]::UtcNow.ToString('O')}
$metadata | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $reportDir 'infrastructure.json') -Encoding utf8
Write-Output ($metadata | Select-Object owner,arm,app,generator | ConvertTo-Json -Compress)
