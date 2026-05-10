param(
    [string]$Exe = "",
    [string]$Bind = "127.0.0.1:47188",
    [int]$TimeoutSeconds = 15
)

$ErrorActionPreference = "Stop"

function Resolve-QorxExe {
    param([string]$Requested)
    if ($Requested) {
        return (Resolve-Path -LiteralPath $Requested).Path
    }
    $candidates = @(
        ".\target\release\qorx.exe",
        ".\target\debug\qorx.exe",
        "qorx"
    )
    foreach ($candidate in $candidates) {
        if (Test-Path -LiteralPath $candidate) {
            return (Resolve-Path -LiteralPath $candidate).Path
        }
    }
    return "qorx"
}

function Get-BaseUrl {
    param([string]$BindValue)
    if ($BindValue.StartsWith("0.0.0.0:")) {
        return "http://127.0.0.1:$($BindValue.Split(':')[-1])"
    }
    return "http://$BindValue"
}

$exePath = Resolve-QorxExe -Requested $Exe
$tempRoot = [System.IO.Path]::GetTempPath()
$qorxHome = Join-Path $tempRoot ("qorx-smoke-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $qorxHome | Out-Null

$oldHome = $env:QORX_HOME
$oldBind = $env:QORX_BIND
$proc = $null

try {
    $env:QORX_HOME = $qorxHome
    $env:QORX_BIND = $Bind
    $base = Get-BaseUrl -BindValue $Bind

    $proc = Start-Process -FilePath $exePath -ArgumentList "daemon" -PassThru -WindowStyle Hidden
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    $health = $null

    while ((Get-Date) -lt $deadline) {
        try {
            $health = Invoke-RestMethod -Uri "$base/health" -TimeoutSec 2
            if ($health.ok -eq $true) {
                break
            }
        } catch {
            Start-Sleep -Milliseconds 250
        }
    }

    if (-not $health -or $health.ok -ne $true) {
        throw "qorx gateway did not become healthy at $base"
    }

    $stats = Invoke-RestMethod -Uri "$base/stats" -TimeoutSec 5
    $doctor = (& $exePath doctor --json | ConvertFrom-Json)
    if ($doctor.bind -ne $Bind) {
        throw "doctor bind mismatch: expected $Bind, got $($doctor.bind)"
    }
    if ($doctor.gateway_healthy -ne $true) {
        throw "doctor reported gateway_healthy=false"
    }

    [pscustomobject]@{
        ok = $true
        version = $health.version
        bind = $Bind
        data_dir = $doctor.data_dir
        requests = $stats.requests
    } | ConvertTo-Json -Depth 4
} finally {
    if ($proc -and -not $proc.HasExited) {
        Stop-Process -Id $proc.Id -Force
    }
    $env:QORX_HOME = $oldHome
    $env:QORX_BIND = $oldBind
    $resolvedHome = Resolve-Path -LiteralPath $qorxHome -ErrorAction SilentlyContinue
    if ($resolvedHome -and $resolvedHome.Path.StartsWith($tempRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        Remove-Item -LiteralPath $resolvedHome.Path -Recurse -Force
    }
}
