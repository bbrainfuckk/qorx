param(
  [double]$Hours = 10,
  [int]$IntervalSeconds = 300,
  [int]$MaxIterations = 0,
  [string]$QorxExe = "$env:USERPROFILE\.cargo\bin\qorx.exe",
  [string]$Repo = (Split-Path -Parent $PSScriptRoot)
)

$ErrorActionPreference = "Continue"
$started = Get-Date
$deadline = $started.AddHours($Hours)
$stamp = $started.ToString("yyyyMMdd-HHmmss")
$runDir = Join-Path $env:LOCALAPPDATA "qorx\Qorx\data\evolve\evolve-$stamp"
New-Item -ItemType Directory -Force -Path $runDir | Out-Null
$log = Join-Path $runDir "evolve.jsonl"
$summary = Join-Path $runDir "summary.json"
$awakeFlags = [Convert]::ToUInt32("80000001", 16) # ES_CONTINUOUS | ES_SYSTEM_REQUIRED

try {
  Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public static class QorxAwake {
  [DllImport("kernel32.dll")]
  public static extern UInt32 SetThreadExecutionState(UInt32 esFlags);
}
"@ -ErrorAction SilentlyContinue
} catch {}

function Request-Awake {
  try {
    [QorxAwake]::SetThreadExecutionState($awakeFlags) | Out-Null
  } catch {}
}

function Write-JsonLine($object) {
  ($object | ConvertTo-Json -Depth 16 -Compress) | Add-Content -LiteralPath $log -Encoding UTF8
}

function Invoke-QorxJson([string[]]$QorxArgs, [string]$QorxHome = $null) {
  $old = $env:QORX_HOME
  try {
    if ($QorxHome) {
      $env:QORX_HOME = $QorxHome
    }
    $stdout = & $QorxExe @QorxArgs 2>&1
    if ($LASTEXITCODE -ne 0) {
      return [pscustomobject]@{ ok = $false; error = ($stdout -join "`n"); value = $null }
    }
    try {
      return [pscustomobject]@{ ok = $true; error = $null; value = (($stdout -join "`n") | ConvertFrom-Json) }
    } catch {
      return [pscustomobject]@{ ok = $true; error = $null; value = ($stdout -join "`n") }
    }
  } catch {
    return [pscustomobject]@{ ok = $false; error = $_.Exception.Message; value = $null }
  } finally {
    $env:QORX_HOME = $old
  }
}

function Estimate-Tokens([string]$Text) {
  if ([string]::IsNullOrEmpty($Text)) { return 0 }
  return [math]::Ceiling($Text.Length / 4.0)
}

function Run-OfficeFreshnessProbe($Iteration) {
  $caseStamp = "$(Get-Date -Format 'yyyyMMdd-HHmmss')-$Iteration"
  $root = Join-Path $env:TEMP "qorx-office-live-$caseStamp"
  $qhome = Join-Path $env:TEMP "qorx-office-home-$caseStamp"
  New-Item -ItemType Directory -Force -Path $root, $qhome | Out-Null
  try {
    @("# Office system", "Stable indexed baseline.") | Set-Content -LiteralPath (Join-Path $root "README.md") -Encoding UTF8
    $oldHome = $env:QORX_HOME
    $env:QORX_HOME = $qhome
    try {
      $indexText = (& $QorxExe index $root 2>&1)
    } finally {
      $env:QORX_HOME = $oldHome
    }
    Start-Sleep -Milliseconds 1200
    '{"db_name":"production_db","port":5432,"status":"deprecated"}' | Set-Content -LiteralPath (Join-Path $root "db_v1_legacy.json") -Encoding UTF8
    '{"db_name":"production_db","port":5433,"status":"active","env":"prod-v2"}' | Set-Content -LiteralPath (Join-Path $root "db_v2_current.json") -Encoding UTF8
    @("env:", "- name: DB_PORT", "  value: `"5433`"") | Set-Content -LiteralPath (Join-Path $root "deploy.yaml") -Encoding UTF8

    $sw = [Diagnostics.Stopwatch]::StartNew()
    $office = Invoke-QorxJson -QorxArgs @("squeeze", "What is DB_PORT in deploy.yaml and active production db port 5433?", "--budget-tokens", "900", "--limit", "5") -QorxHome $qhome
    $sw.Stop()
    $paths = @()
    if ($office.ok -and $office.value.evidence) {
      $paths = @($office.value.evidence | ForEach-Object { $_.path })
    }
    $baseline = Estimate-Tokens ((Get-ChildItem -LiteralPath $root -File -Recurse | ForEach-Object { Get-Content -LiteralPath $_.FullName -Raw }) -join "`n")
    $sent = if ($office.ok -and $office.value.used_tokens) { [int64]$office.value.used_tokens } else { 0 }
    $saved = [math]::Max(0, $baseline - $sent)
    return [pscustomobject]@{
      ok = ($office.ok -and ($paths -contains "deploy.yaml") -and ($paths -contains "db_v2_current.json"))
      ms = [math]::Round($sw.Elapsed.TotalMilliseconds, 2)
      evidence_paths = $paths
      baseline_tokens = $baseline
      sent_tokens = $sent
      saved_tokens = $saved
      estimated_usd_saved = [math]::Round($saved * 2.5 / 1000000.0, 6)
      error = $office.error
      index = ($indexText -join "`n")
    }
  } finally {
    Remove-Item -LiteralPath $root -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $qhome -Recurse -Force -ErrorAction SilentlyContinue
  }
}

function Run-SpamNeedleProbe($Iteration) {
  $caseStamp = "$(Get-Date -Format 'yyyyMMdd-HHmmss')-$Iteration"
  $root = Join-Path $env:TEMP "qorx-spam-live-$caseStamp"
  $qhome = Join-Path $env:TEMP "qorx-spam-home-$caseStamp"
  $spam = Join-Path $root "spam_test"
  New-Item -ItemType Directory -Force -Path $spam, $qhome | Out-Null
  try {
    "stable root" | Set-Content -LiteralPath (Join-Path $root "README.md") -Encoding UTF8
    $oldHome = $env:QORX_HOME
    $env:QORX_HOME = $qhome
    try {
      $indexText = (& $QorxExe index $root 2>&1)
    } finally {
      $env:QORX_HOME = $oldHome
    }
    Start-Sleep -Milliseconds 1200
    1..500 | ForEach-Object {
      $content = if ($_ -eq 404) { "const CRITICAL_SYSTEM_PASSWORD = `"PASSWORD_STRESS_TEST_999`";" } else { "export function doWork() { return `"ok`"; }" }
      Set-Content -LiteralPath (Join-Path $spam "spam_$_.js") -Value $content -Encoding UTF8
    }
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $answer = Invoke-QorxJson -QorxArgs @("strict-answer", "Where is CRITICAL_SYSTEM_PASSWORD PASSWORD_STRESS_TEST_999?", "--limit", "4") -QorxHome $qhome
    $sw.Stop()
    $paths = @()
    if ($answer.ok -and $answer.value.evidence) {
      $paths = @($answer.value.evidence | ForEach-Object { $_.path })
    }
    $baseline = Estimate-Tokens ((Get-ChildItem -LiteralPath $root -File -Recurse | ForEach-Object { Get-Content -LiteralPath $_.FullName -Raw }) -join "`n")
    $sent = if ($answer.ok -and $answer.value.used_tokens) { [int64]$answer.value.used_tokens } else { 0 }
    $saved = [math]::Max(0, $baseline - $sent)
    return [pscustomobject]@{
      ok = ($answer.ok -and ($paths -contains "spam_test/spam_404.js"))
      ms = [math]::Round($sw.Elapsed.TotalMilliseconds, 2)
      coverage = if ($answer.ok) { $answer.value.coverage } else { "error" }
      evidence_paths = $paths
      baseline_tokens = $baseline
      sent_tokens = $sent
      saved_tokens = $saved
      estimated_usd_saved = [math]::Round($saved * 2.5 / 1000000.0, 6)
      error = $answer.error
      index = ($indexText -join "`n")
    }
  } finally {
    Remove-Item -LiteralPath $root -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $qhome -Recurse -Force -ErrorAction SilentlyContinue
  }
}

$iteration = 0
$failures = 0
Request-Awake
Write-JsonLine ([pscustomobject]@{
  event = "start"
  started_at = $started.ToString("o")
  deadline = $deadline.ToString("o")
  repo = $Repo
  qorx = $QorxExe
  log = $log
  max_iterations = $MaxIterations
})

while (((Get-Date) -lt $deadline) -and (($MaxIterations -le 0) -or ($iteration -lt $MaxIterations))) {
  Request-Awake
  $iteration++
  $iterStart = Get-Date
  $doctor = Invoke-QorxJson -QorxArgs @("doctor", "--json")
  $stats = Invoke-QorxJson -QorxArgs @("stats")
  $office = Run-OfficeFreshnessProbe $iteration
  $spam = Run-SpamNeedleProbe $iteration
  $liveTests = $null
  if (($iteration -eq 1) -or ($iteration % 12 -eq 0)) {
    Push-Location $Repo
    try {
      $out = (& cargo test live_overlay 2>&1)
      $liveTests = [pscustomobject]@{ ok = ($LASTEXITCODE -eq 0); output_tail = (($out | Select-Object -Last 20) -join "`n") }
    } finally {
      Pop-Location
    }
  }
  $ok = $doctor.ok -and $office.ok -and $spam.ok -and (($null -eq $liveTests) -or $liveTests.ok)
  if (-not $ok) { $failures++ }
  Write-JsonLine ([pscustomobject]@{
    event = "iteration"
    iteration = $iteration
    timestamp = $iterStart.ToString("o")
    ok = $ok
    doctor_ok = $doctor.ok
    office = $office
    spam = $spam
    stats_ok = $stats.ok
    stats = $stats.value
    live_overlay_tests = $liveTests
  })
  if (((Get-Date) -lt $deadline) -and (($MaxIterations -le 0) -or ($iteration -lt $MaxIterations))) {
    Start-Sleep -Seconds $IntervalSeconds
  }
}

try {
  [QorxAwake]::SetThreadExecutionState([Convert]::ToUInt32("80000000", 16)) | Out-Null
} catch {}

$finished = Get-Date
$summaryObject = [pscustomobject]@{
  schema = "qorx.overnight-evolve.v1"
  started_at = $started.ToString("o")
  finished_at = $finished.ToString("o")
  iterations = $iteration
  failures = $failures
  ok = ($failures -eq 0)
  max_iterations = $MaxIterations
  log = $log
}
$summaryObject | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $summary -Encoding UTF8
Write-JsonLine ([pscustomobject]@{ event = "finish"; summary = $summaryObject })
