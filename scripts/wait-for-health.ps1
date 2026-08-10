# Poll a URL until it returns a 2xx, or fail after a bounded timeout. Windows
# counterpart of wait-for-health.sh (same arguments, same exit codes), for a
# locally-run mw-server on a machine without curl/sh.
#
# Usage: pwsh scripts/wait-for-health.ps1 <url> [timeout_seconds]
# Example: pwsh scripts/wait-for-health.ps1 http://localhost:8080/healthz 120
#
# NOTE: the CI steps that call the .sh form all wait on docker-compose services
# and stay Linux-only by design; this twin exists for local Windows development.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Url,
    [int]$TimeoutSeconds = 120
)
$ErrorActionPreference = 'Stop'

$deadline = (Get-Date).AddSeconds($TimeoutSeconds)
Write-Host "waiting for $Url (timeout ${TimeoutSeconds}s)..."

while ($true) {
    try {
        # -UseBasicParsing keeps this off the IE engine on Windows PowerShell 5.1.
        $resp = Invoke-WebRequest -Uri $Url -UseBasicParsing -TimeoutSec 5 -Method Get
        if ($resp.StatusCode -ge 200 -and $resp.StatusCode -lt 300) {
            Write-Host "healthy: $Url"
            exit 0
        }
    } catch {
        # Not up yet (connection refused, 5xx, DNS) — keep polling until the deadline.
    }
    if ((Get-Date) -ge $deadline) {
        Write-Error "TIMEOUT waiting for $Url after ${TimeoutSeconds}s"
        exit 1
    }
    Start-Sleep -Seconds 2
}
