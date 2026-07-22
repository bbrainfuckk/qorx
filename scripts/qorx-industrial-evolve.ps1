param(
  [int]$MaxScenarios = 7,
  [int]$IntervalSeconds = 1,
  [string]$QorxExe = "$env:USERPROFILE\.cargo\bin\qorx.exe",
  [string]$Repo = (Split-Path -Parent $PSScriptRoot)
)

$ErrorActionPreference = "Continue"
$started = Get-Date
$stamp = $started.ToString("yyyyMMdd-HHmmss")
$runDir = Join-Path $env:LOCALAPPDATA "qorx\Qorx\data\evolve\industrial-$stamp"
New-Item -ItemType Directory -Force -Path $runDir | Out-Null
$log = Join-Path $runDir "industrial-evolve.jsonl"
$summary = Join-Path $runDir "summary.json"
$pricingUsdPerMillion = 2.5

$scienceBasis = @(
  "Lost in the Middle (arXiv:2307.03172): long-context systems need position and recall tests, not just larger context.",
  "RAGAS (arXiv:2309.15217): evaluate context relevance, context precision, answer support, and faithfulness separately.",
  "RepoBench (arXiv:2306.03091): code retrieval must be tested at repository and multi-file scale.",
  "Self-RAG (arXiv:2310.11511): retrieval systems need critique/verification steps when evidence is weak or conflicting."
)

function Write-JsonLine($object) {
  ($object | ConvertTo-Json -Depth 18 -Compress) | Add-Content -LiteralPath $log -Encoding UTF8
}

function Invoke-QorxJson([string[]]$QorxArgs, [string]$QorxHome = $null) {
  $old = $env:QORX_HOME
  try {
    if ($QorxHome) { $env:QORX_HOME = $QorxHome }
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

function Get-BaselineTokens([string]$Root) {
  $text = (Get-ChildItem -LiteralPath $Root -File -Recurse | ForEach-Object {
    Get-Content -LiteralPath $_.FullName -Raw
  }) -join "`n"
  Estimate-Tokens $text
}

function Get-EvidencePaths($Result) {
  if ($Result.ok -and $Result.value.evidence) {
    return @($Result.value.evidence | ForEach-Object { $_.path })
  }
  return @()
}

function New-Case([string]$Name, [int]$Iteration) {
  $caseStamp = "$(Get-Date -Format 'yyyyMMdd-HHmmss')-$Iteration"
  $root = Join-Path $env:TEMP "qorx-industrial-$Name-$caseStamp"
  $qhome = Join-Path $env:TEMP "qorx-industrial-home-$Name-$caseStamp"
  New-Item -ItemType Directory -Force -Path $root, $qhome | Out-Null
  [pscustomobject]@{ root = $root; qhome = $qhome }
}

function Index-Baseline($Case) {
  "stable indexed baseline" | Set-Content -LiteralPath (Join-Path $Case.root "README.md") -Encoding UTF8
  return Index-Current $Case
}

function Index-Current($Case) {
  $old = $env:QORX_HOME
  $env:QORX_HOME = $Case.qhome
  try {
    return (& $QorxExe index $Case.root 2>&1) -join "`n"
  } finally {
    $env:QORX_HOME = $old
  }
}

function New-ScenarioResult($Name, $Description, $Result, [string]$Root, [string[]]$ExpectedPaths, [string[]]$Gaps, [bool]$ProductionReady) {
  $paths = @(Get-EvidencePaths $Result)
  $missing = @($ExpectedPaths | Where-Object { $paths -notcontains $_ })
  $baseline = Get-BaselineTokens $Root
  $sent = if ($Result.ok -and $Result.value.used_tokens) { [int64]$Result.value.used_tokens } else { 0 }
  $saved = [math]::Max(0, $baseline - $sent)
  $ok = ($Result.ok -and $missing.Count -eq 0)
  $ready = ($ProductionReady -and $ok -and $Gaps.Count -eq 0)
  [pscustomobject]@{
    scenario = $Name
    description = $Description
    ok = $ok
    production_ready = $ready
    expected_paths = @($ExpectedPaths)
    evidence_paths = @($paths)
    missing_paths = @($missing)
    baseline_tokens = $baseline
    sent_tokens = $sent
    saved_tokens = $saved
    estimated_usd_saved = [math]::Round($saved * $pricingUsdPerMillion / 1000000.0, 6)
    gaps = @($Gaps)
    error = $Result.error
  }
}

function Run-GraphTopologyAudit([int]$Iteration) {
  $case = New-Case "graph-topology" $Iteration
  try {
    $src = Join-Path $case.root "src"
    $docs = Join-Path $case.root "docs"
    New-Item -ItemType Directory -Force -Path $src, $docs | Out-Null
    @(
      "pub fn charge_card() {}",
      "pub fn refund_card() {}"
    ) -join "`n" | Set-Content -LiteralPath (Join-Path $src "payments.rs") -Encoding UTF8
    @(
      "pub fn audit_payment() {}",
      "pub fn audit_refund() {}"
    ) -join "`n" | Set-Content -LiteralPath (Join-Path $src "audit.rs") -Encoding UTF8
    @(
      "pub fn handle_checkout() {",
      "    charge_card();",
      "    audit_payment();",
      "}",
      "pub fn handle_refund() {",
      "    refund_card();",
      "    audit_refund();",
      "}"
    ) -join "`n" | Set-Content -LiteralPath (Join-Path $src "routes.rs") -Encoding UTF8
    "Runbook only; no code references yet." | Set-Content -LiteralPath (Join-Path $docs "runbook.md") -Encoding UTF8
    $index = Index-Current $case
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $result = Invoke-QorxJson -QorxArgs @("graph", "--limit", "128") -QorxHome $case.qhome
    $sw.Stop()
    $gaps = @()
    $hotPaths = @()
    if ($result.ok -and $result.value.metrics) {
      $hotPaths = @($result.value.metrics.top_referenced_files | ForEach-Object { $_.path })
      if ([int]$result.value.metrics.reference_edges -lt 2) {
        $gaps += "graph-recall: expected routes.rs to reference payment and audit service files."
      }
      if ([int]$result.value.metrics.isolated_files -lt 1) {
        $gaps += "graph-criticism: isolated runbook file was not surfaced as an architecture gap."
      }
      if ($hotPaths -notcontains "src/payments.rs") {
        $gaps += "graph-hotspot: payment service did not appear in top referenced files."
      }
      if ($hotPaths -notcontains "src/audit.rs") {
        $gaps += "graph-hotspot: audit service did not appear in top referenced files."
      }
    } else {
      $gaps += "graph-command: qorx graph did not return metric JSON."
    }
    $baseline = Get-BaselineTokens $case.root
    $graphJson = if ($result.ok) { $result.value | ConvertTo-Json -Depth 18 -Compress } else { "" }
    $sent = Estimate-Tokens $graphJson
    $saved = [math]::Max(0, $baseline - $sent)
    $ok = ($result.ok -and $gaps.Count -eq 0)
    return [pscustomobject]@{
      scenario = "graph_topology_audit"
      description = "Static code topology audit with reference hotspots and isolated-file criticism."
      ok = $ok
      production_ready = $ok
      expected_paths = @("src/payments.rs", "src/audit.rs", "docs/runbook.md")
      evidence_paths = @($hotPaths)
      missing_paths = @()
      baseline_tokens = $baseline
      sent_tokens = $sent
      saved_tokens = $saved
      estimated_usd_saved = [math]::Round($saved * $pricingUsdPerMillion / 1000000.0, 6)
      gaps = @($gaps)
      error = $result.error
      ms = [math]::Round($sw.Elapsed.TotalMilliseconds, 2)
      index = $index
      metrics = if ($result.ok) { $result.value.metrics } else { $null }
    }
  } finally {
    Remove-Item -LiteralPath $case.root, $case.qhome -Recurse -Force -ErrorAction SilentlyContinue
  }
}

function Run-GraphPathTraceAudit([int]$Iteration) {
  $case = New-Case "graph-path-trace" $Iteration
  try {
    $src = Join-Path $case.root "src"
    New-Item -ItemType Directory -Force -Path $src | Out-Null
    "// route layer`npub fn handle_checkout() { process_invoice(); }" | Set-Content -LiteralPath (Join-Path $src "routes.rs") -Encoding UTF8
    "// billing layer`npub fn process_invoice() { write_ledger(); }" | Set-Content -LiteralPath (Join-Path $src "billing.rs") -Encoding UTF8
    "// ledger layer`npub fn write_ledger() {}" | Set-Content -LiteralPath (Join-Path $src "ledger.rs") -Encoding UTF8
    $index = Index-Current $case
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $result = Invoke-QorxJson -QorxArgs @("graph-path", "routes.rs", "ledger.rs", "--limit", "128") -QorxHome $case.qhome
    $sw.Stop()
    $gaps = @()
    $tracePaths = @()
    if ($result.ok -and $result.value.found) {
      $tracePaths = @($result.value.path | ForEach-Object { $_.path })
      if ([int]$result.value.hops -ne 2) {
        $gaps += "graph-path: expected two-hop route from routes.rs to ledger.rs through billing.rs."
      }
      foreach ($expected in @("src/routes.rs", "src/billing.rs", "src/ledger.rs")) {
        if ($tracePaths -notcontains $expected) {
          $gaps += "graph-path: missing trace step $expected."
        }
      }
    } else {
      $gaps += "graph-path: no extracted path from checkout route to ledger write."
    }
    $baseline = Get-BaselineTokens $case.root
    $traceJson = if ($result.ok) { $result.value | ConvertTo-Json -Depth 12 -Compress } else { "" }
    $sent = Estimate-Tokens $traceJson
    $saved = [math]::Max(0, $baseline - $sent)
    $ok = ($result.ok -and $gaps.Count -eq 0)
    return [pscustomobject]@{
      scenario = "graph_path_trace_audit"
      description = "Production incident trace from route handler to downstream ledger write."
      ok = $ok
      production_ready = $ok
      expected_paths = @("src/routes.rs", "src/billing.rs", "src/ledger.rs")
      evidence_paths = @($tracePaths)
      missing_paths = @()
      baseline_tokens = $baseline
      sent_tokens = $sent
      saved_tokens = $saved
      estimated_usd_saved = [math]::Round($saved * $pricingUsdPerMillion / 1000000.0, 6)
      gaps = @($gaps)
      error = $result.error
      ms = [math]::Round($sw.Elapsed.TotalMilliseconds, 2)
      index = $index
      trace = if ($result.ok) { $result.value } else { $null }
    }
  } finally {
    Remove-Item -LiteralPath $case.root, $case.qhome -Recurse -Force -ErrorAction SilentlyContinue
  }
}

function Run-OfficeConfigFreshness([int]$Iteration) {
  $case = New-Case "office-config" $Iteration
  try {
    $index = Index-Baseline $case
    Start-Sleep -Milliseconds 1200
    '{"db_name":"production_db","port":5432,"status":"deprecated"}' | Set-Content -LiteralPath (Join-Path $case.root "db_v1_legacy.json") -Encoding UTF8
    '{"db_name":"production_db","port":5433,"status":"active","env":"prod-v2"}' | Set-Content -LiteralPath (Join-Path $case.root "db_v2_current.json") -Encoding UTF8
    @("env:", "- name: DB_PORT", "  value: `"5433`"") | Set-Content -LiteralPath (Join-Path $case.root "deploy.yaml") -Encoding UTF8
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $result = Invoke-QorxJson -QorxArgs @("squeeze", "Resolve active production DB_PORT. Prefer deploy.yaml and active prod-v2 over deprecated 5432.", "--budget-tokens", "900", "--limit", "8") -QorxHome $case.qhome
    $sw.Stop()
    $paths = Get-EvidencePaths $result
    $gaps = @()
    if ($paths -contains "db_v1_legacy.json") { $gaps += "authority-ranking: deprecated source still appears beside current deploy truth." }
    $base = New-ScenarioResult "office_config_freshness" "Fresh deployment config conflict after baseline index." $result $case.root @("deploy.yaml", "db_v2_current.json") $gaps ($gaps.Count -eq 0)
    $base | Add-Member -NotePropertyName ms -NotePropertyValue ([math]::Round($sw.Elapsed.TotalMilliseconds, 2))
    $base | Add-Member -NotePropertyName index -NotePropertyValue $index
    return $base
  } finally {
    Remove-Item -LiteralPath $case.root, $case.qhome -Recurse -Force -ErrorAction SilentlyContinue
  }
}

function Run-LegacyApiRecall([int]$Iteration) {
  $case = New-Case "legacy-api" $Iteration
  try {
    $index = Index-Baseline $case
    Start-Sleep -Milliseconds 1200
    $dir = Join-Path $case.root "microservices"
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $needles = @(14, 37, 52, 88, 91)
    1..120 | ForEach-Object {
      $content = @(
        "// Microservice $_ handler",
        "function handleRequest(req, res) {",
        "  const requestId = req.id;",
        ($(if ($needles -contains $_) { "  const data = legacyDatabase.fetchSync_DEPRECATED(requestId);" } else { "  const data = modernDb.fetchAsync(requestId);" })),
        "  res.send(data);",
        "}"
      ) -join "`n"
      Set-Content -LiteralPath (Join-Path $dir "service_$_.js") -Value $content -Encoding UTF8
    }
    $expected = @($needles | ForEach-Object { "microservices/service_$_.js" })
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $result = Invoke-QorxJson -QorxArgs @("squeeze", "Find every fetchSync_DEPRECATED legacyDatabase usage in microservices.", "--budget-tokens", "900", "--limit", "12") -QorxHome $case.qhome
    $sw.Stop()
    $paths = Get-EvidencePaths $result
    $missing = @($expected | Where-Object { $paths -notcontains $_ })
    $gaps = @()
    if ($missing.Count -gt 0) { $gaps += "recall-gap: missed deprecated API usages: $($missing -join ', ')." }
    $base = New-ScenarioResult "legacy_api_recall" "Multi-file deprecated API audit across generated services." $result $case.root $expected $gaps ($gaps.Count -eq 0)
    $base | Add-Member -NotePropertyName ms -NotePropertyValue ([math]::Round($sw.Elapsed.TotalMilliseconds, 2))
    $base | Add-Member -NotePropertyName index -NotePropertyValue $index
    return $base
  } finally {
    Remove-Item -LiteralPath $case.root, $case.qhome -Recurse -Force -ErrorAction SilentlyContinue
  }
}

function Run-IncidentSecretNeedle([int]$Iteration) {
  $case = New-Case "incident-secret" $Iteration
  try {
    $index = Index-Baseline $case
    Start-Sleep -Milliseconds 1200
    $dir = Join-Path $case.root "logs"
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    1..750 | ForEach-Object {
      $line = if ($_ -eq 617) {
        "WARN incident_id=INC-617 PAYMENT_TOKEN_LIVE=tok_live_redacted CUSTOMER_SSN=999-12-3456 export_blocked=true"
      } else {
        "INFO incident_id=INC-$_ request completed without payment or identity anomaly"
      }
      Set-Content -LiteralPath (Join-Path $dir "app_$_.log") -Value $line -Encoding UTF8
    }
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $result = Invoke-QorxJson -QorxArgs @("squeeze", "Which log contains PAYMENT_TOKEN_LIVE CUSTOMER_SSN incident evidence?", "--budget-tokens", "900", "--limit", "4") -QorxHome $case.qhome
    $sw.Stop()
    $gaps = @()
    $base = New-ScenarioResult "incident_secret_needle" "P1 security scan over 750 fresh log files." $result $case.root @("logs/app_617.log") $gaps $true
    $base | Add-Member -NotePropertyName ms -NotePropertyValue ([math]::Round($sw.Elapsed.TotalMilliseconds, 2))
    $base | Add-Member -NotePropertyName index -NotePropertyValue $index
    return $base
  } finally {
    Remove-Item -LiteralPath $case.root, $case.qhome -Recurse -Force -ErrorAction SilentlyContinue
  }
}

function Run-PolicyConflictAuthority([int]$Iteration) {
  $case = New-Case "policy-conflict" $Iteration
  try {
    $index = Index-Baseline $case
    Start-Sleep -Milliseconds 1200
    $dir = Join-Path $case.root "policy"
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    "status: superseded`nretention_days: 365`napproved_by: old-council" | Set-Content -LiteralPath (Join-Path $dir "retention_2024.md") -Encoding UTF8
    "status: active`nretention_days: 30`napproved_by: DPO`neffective: 2026-05-01" | Set-Content -LiteralPath (Join-Path $dir "retention_2026.md") -Encoding UTF8
    "const RETENTION_DAYS = 365; // stale fallback from old policy" | Set-Content -LiteralPath (Join-Path $case.root "exporter.js") -Encoding UTF8
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $result = Invoke-QorxJson -QorxArgs @("squeeze", "What is the active retention policy and retention_days? Prefer status active over superseded or stale code.", "--budget-tokens", "900", "--limit", "8") -QorxHome $case.qhome
    $sw.Stop()
    $paths = Get-EvidencePaths $result
    $gaps = @()
    if (($paths -contains "policy/retention_2024.md") -or ($paths -contains "exporter.js")) {
      $gaps += "authority-ranking: stale policy/code appears with active policy; needs stronger source-of-truth scoring."
    }
    $base = New-ScenarioResult "policy_conflict_authority" "Conflicting compliance policy and stale code fallback." $result $case.root @("policy/retention_2026.md") $gaps ($gaps.Count -eq 0)
    $base | Add-Member -NotePropertyName ms -NotePropertyValue ([math]::Round($sw.Elapsed.TotalMilliseconds, 2))
    $base | Add-Member -NotePropertyName index -NotePropertyValue $index
    return $base
  } finally {
    Remove-Item -LiteralPath $case.root, $case.qhome -Recurse -Force -ErrorAction SilentlyContinue
  }
}

function Run-SemanticAliasInfra([int]$Iteration) {
  $case = New-Case "semantic-alias" $Iteration
  try {
    $index = Index-Baseline $case
    Start-Sleep -Milliseconds 1200
    $dir = Join-Path $case.root "ops"
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    "SERVICE_GATEWAY_ENTRANCE=5433`nprod-v2 ingress opened" | Set-Content -LiteralPath (Join-Path $dir "gateway_entrance.conf") -Encoding UTF8
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $result = Invoke-QorxJson -QorxArgs @("squeeze", "What DB_PORT should production connect to?", "--budget-tokens", "900", "--limit", "4") -QorxHome $case.qhome
    $sw.Stop()
    $gaps = @()
    $base = New-ScenarioResult "semantic_alias_infra" "Business synonym query: DB_PORT must resolve to SERVICE_GATEWAY_ENTRANCE in .conf infra config." $result $case.root @("ops/gateway_entrance.conf") $gaps $true
    $base | Add-Member -NotePropertyName ms -NotePropertyValue ([math]::Round($sw.Elapsed.TotalMilliseconds, 2))
    $base | Add-Member -NotePropertyName index -NotePropertyValue $index
    return $base
  } finally {
    Remove-Item -LiteralPath $case.root, $case.qhome -Recurse -Force -ErrorAction SilentlyContinue
  }
}

$scenarios = @(
  ${function:Run-GraphTopologyAudit},
  ${function:Run-GraphPathTraceAudit},
  ${function:Run-OfficeConfigFreshness},
  ${function:Run-LegacyApiRecall},
  ${function:Run-IncidentSecretNeedle},
  ${function:Run-PolicyConflictAuthority},
  ${function:Run-SemanticAliasInfra}
)

Write-JsonLine ([pscustomobject]@{
  event = "start"
  schema = "qorx.industrial-evolve.v1"
  started_at = $started.ToString("o")
  repo = $Repo
  qorx = $QorxExe
  max_scenarios = $MaxScenarios
  science_basis = $scienceBasis
  log = $log
})

$results = @()
$limit = [math]::Min($MaxScenarios, $scenarios.Count)
for ($i = 0; $i -lt $limit; $i++) {
  $iteration = $i + 1
  $scenario = & $scenarios[$i] $iteration
  $results += $scenario
  Write-JsonLine ([pscustomobject]@{ event = "scenario"; iteration = $iteration; result = $scenario })
  if ($iteration -lt $limit) {
    Start-Sleep -Seconds $IntervalSeconds
  }
}

$failures = @($results | Where-Object { -not $_.ok })
$notReady = @($results | Where-Object { -not $_.production_ready })
$allGaps = @($results | ForEach-Object { $_.gaps } | Where-Object { $_ })
$summaryObject = [pscustomobject]@{
  schema = "qorx.industrial-evolve-summary.v1"
  started_at = $started.ToString("o")
  finished_at = (Get-Date).ToString("o")
  scenarios = $results.Count
  failures = $failures.Count
  production_ready = @($results | Where-Object { $_.production_ready }).Count
  gaps_found = $allGaps
  total_saved_tokens = (($results | ForEach-Object { [int64]$_.saved_tokens }) | Measure-Object -Sum).Sum
  total_estimated_usd_saved = [math]::Round((($results | ForEach-Object { [double]$_.estimated_usd_saved }) | Measure-Object -Sum).Sum, 6)
  science_basis = $scienceBasis
  log = $log
}

$summaryObject | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $summary -Encoding UTF8
Write-JsonLine ([pscustomobject]@{ event = "finish"; summary = $summaryObject })
$summaryObject | ConvertTo-Json -Depth 12
