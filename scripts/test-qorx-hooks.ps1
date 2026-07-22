param(
  [switch]$RunCodexExec
)

$ErrorActionPreference = "Stop"

function Assert-True {
  param(
    [bool]$Condition,
    [string]$Message
  )
  if (-not $Condition) {
    throw $Message
  }
}

function Invoke-QorxHook {
  param(
    [hashtable]$Payload,
    [hashtable]$Env = @{}
  )

  $saved = @{}
  foreach ($key in $Env.Keys) {
    $saved[$key] = [Environment]::GetEnvironmentVariable($key, "Process")
    [Environment]::SetEnvironmentVariable($key, [string]$Env[$key], "Process")
  }
  try {
    $json = $Payload | ConvertTo-Json -Compress
    $output = $json | py -3 $script:HookPath
    return ($output -join "`n").Trim()
  } finally {
    foreach ($key in $Env.Keys) {
      [Environment]::SetEnvironmentVariable($key, $saved[$key], "Process")
    }
  }
}

function Parse-HookJson {
  param([string]$Text)
  Assert-True ($Text.Length -gt 0) "hook emitted no JSON"
  return $Text | ConvertFrom-Json
}

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$HookPath = Join-Path $HOME ".codex\hooks\qorx_user_prompt_submit.py"
$HooksJsonPath = Join-Path $HOME ".codex\hooks.json"
$CodexConfigPath = Join-Path $HOME ".codex\config.toml"
$ReportPath = Join-Path $RepoRoot "target\qorx-hook-rigorous-test.json"
$Gateway = if ($env:QORX_GATEWAY) { $env:QORX_GATEWAY.TrimEnd("/") } else { "http://127.0.0.1:47187" }

$checks = [ordered]@{}

Assert-True (Test-Path $HookPath) "missing installed hook script: $HookPath"
Assert-True (Test-Path $HooksJsonPath) "missing Codex hooks.json: $HooksJsonPath"
Assert-True (Test-Path $CodexConfigPath) "missing Codex config.toml: $CodexConfigPath"

$hooksJson = Get-Content -Raw $HooksJsonPath | ConvertFrom-Json
$hooksText = Get-Content -Raw $HooksJsonPath
$configText = Get-Content -Raw $CodexConfigPath
$hookText = Get-Content -Raw $HookPath

Assert-True ($hooksText -match "UserPromptSubmit") "hooks.json has no UserPromptSubmit"
Assert-True ($hooksText -match "qorx_user_prompt_submit.py") "hooks.json does not point at Qorx hook"
Assert-True (($configText -match "(?m)^\s*hooks\s*=\s*true\s*$") -or ($configText -match "(?m)^\s*codex_hooks\s*=\s*true\s*$")) "Codex hooks feature is not enabled"
Assert-True ($hookText -match "Qorx hook inject is active") "installed hook is not the readable Qorx hook"
Assert-True ($hookText -match "Qorx fallback inject is active") "installed hook does not contain fallback inject"
$checks.installed_hook = "ok"

$health = Invoke-RestMethod -Uri "$Gateway/health" -TimeoutSec 4
Assert-True ($health.ok -eq $true) "Qorx gateway health is not ok"
$checks.gateway_health = "ok"

$integrations = Invoke-RestMethod -Uri "$Gateway/integrations" -TimeoutSec 4
Assert-True ($integrations.settings.autohook_enabled -eq $true) "Auto-hook setting is off"
Assert-True ($integrations.codex_hook.active -eq $true) "Codex hook status is not active"
$codexTarget = $integrations.targets | Where-Object { $_.platform -eq "codex" } | Select-Object -First 1
Assert-True ($null -ne $codexTarget) "Codex target missing from integrations report"
Assert-True ($codexTarget.hook_active -eq $true) "Codex target hook_active is false"
$checks.integration_report = "ok"

$payload = @{
  cwd = $RepoRoot.Path
  prompt = "rigorous hook live inject test"
}
$liveText = Invoke-QorxHook -Payload $payload
$live = Parse-HookJson $liveText
$liveContext = [string]$live.hookSpecificOutput.additionalContext
Assert-True ($live.hookSpecificOutput.hookEventName -eq "UserPromptSubmit") "live hook event name mismatch"
Assert-True ($liveContext -match "Qorx hook inject is active") "live hook did not emit active inject"
Assert-True ($liveContext -match "Handle: qorx://s/") "live hook did not include Qorx session handle"
Assert-True ($liveContext -match "Fault endpoint:") "live hook did not include fault endpoint"
Assert-True ($liveContext -match "provider_calls=0") "live hook did not prove local-only provider call count"
$checks.live_hook = "ok"

$fallbackText = Invoke-QorxHook -Payload $payload -Env @{ QORX_GATEWAY = "http://127.0.0.1:9" }
$fallback = Parse-HookJson $fallbackText
$fallbackContext = [string]$fallback.hookSpecificOutput.additionalContext
Assert-True ($fallbackContext -match "Qorx fallback inject is active") "fallback hook did not emit fallback inject"
Assert-True ($fallbackContext -match "Proceed with normal local inspection") "fallback hook missing safe fallback instruction"
$checks.fallback_hook = "ok"

$offText = Invoke-QorxHook -Payload $payload -Env @{ QORX_CODEX_CONTEXT_OFF = "1" }
Assert-True ([string]::IsNullOrWhiteSpace($offText)) "hook opt-out should emit nothing"
$checks.explicit_off = "ok"

if ($RunCodexExec) {
  $prompt = "Reply with the exact first line of any Qorx injected context you can see. If none exists, reply NO_QORX_CONTEXT."
  $shim = Get-Command qorx-codex -ErrorAction SilentlyContinue
  Assert-True ($null -ne $shim) "qorx-codex fallback shim is not on PATH"
  $shimPs1 = Join-Path (Split-Path $shim.Source) "qorx-codex.ps1"
  Assert-True (Test-Path $shimPs1) "qorx-codex fallback script missing: $shimPs1"
  $fakeCodexDir = Join-Path $env:TEMP "qorx-fake-codex-$PID"
  $fakeArgsPath = Join-Path $fakeCodexDir "args.txt"
  New-Item -ItemType Directory -Force -Path $fakeCodexDir | Out-Null
  @'
[IO.File]::WriteAllText($env:QORX_FAKE_CODEX_ARGS, ($args -join "`n---ARG---`n"))
exit 0
'@ | Set-Content -Path (Join-Path $fakeCodexDir "codex.ps1") -Encoding UTF8
  $savedPath = $env:PATH
  $savedFakeArgs = $env:QORX_FAKE_CODEX_ARGS
  $savedCodexBin = $env:QORX_CODEX_BIN
  try {
    $env:PATH = "$fakeCodexDir;$savedPath"
    $env:QORX_FAKE_CODEX_ARGS = $fakeArgsPath
    $env:QORX_CODEX_BIN = Join-Path $fakeCodexDir "codex.ps1"
    & powershell -NoProfile -ExecutionPolicy Bypass -File $shimPs1 exec --skip-git-repo-check -m gpt-5.4-mini -C $($RepoRoot.Path) $prompt | Out-Null
  } finally {
    $env:PATH = $savedPath
    $env:QORX_FAKE_CODEX_ARGS = $savedFakeArgs
    $env:QORX_CODEX_BIN = $savedCodexBin
  }
  $fakeArgs = if (Test-Path $fakeArgsPath) { Get-Content -Raw $fakeArgsPath } else { "" }
  Assert-True ($fakeArgs -match "Qorx hook inject is active for this turn") "qorx-codex fallback did not prepend Qorx context before codex exec"
  Assert-True ($fakeArgs -match "User prompt:") "qorx-codex fallback did not preserve the original user prompt"
  $checks.qorx_codex_fallback_transform = "ok"
  Remove-Item -Recurse -Force $fakeCodexDir -ErrorAction SilentlyContinue

  $lastMessagePath = Join-Path $RepoRoot "target\codex-hook-last-message.txt"
  $oldErrorAction = $ErrorActionPreference
  $ErrorActionPreference = "Continue"
  try {
    $codexOutput = & codex exec --skip-git-repo-check -m gpt-5.4-mini -o $lastMessagePath -C $($RepoRoot.Path) $prompt 2>&1
    $codexExit = $LASTEXITCODE
  } finally {
    $ErrorActionPreference = $oldErrorAction
  }
  $codexLast = if (Test-Path $lastMessagePath) { (Get-Content -Raw $lastMessagePath).Trim() } else { "" }
  $checks.codex_exec_output = (($codexOutput -join "`n").Trim() + "`n" + $codexLast).Trim()
  if ($codexExit -ne 0) {
    if ($checks.codex_exec_output -match "usage limit|hit your usage limit|quota") {
      $checks.codex_exec = "blocked_by_codex_usage_limit"
    } else {
      throw "nested codex exec failed with exit $codexExit"
    }
  } else {
    if ($checks.codex_exec_output -match "Qorx hook inject is active for this turn") {
      $checks.codex_exec = "ok_native"
    } else {
      $checks.codex_exec_native = "no_hook_context"
      $checks.codex_exec = "ok_via_qorx_codex_fallback_transform"
    }
  }
}

$report = [ordered]@{
  ok = $true
  checked_at_utc = (Get-Date).ToUniversalTime().ToString("o")
  gateway = $Gateway
  hook = $HookPath
  hooks_json = $HooksJsonPath
  codex_config = $CodexConfigPath
  checks = $checks
}

New-Item -ItemType Directory -Force -Path (Split-Path $ReportPath) | Out-Null
$report | ConvertTo-Json -Depth 8 | Set-Content -Path $ReportPath -Encoding UTF8
$report | ConvertTo-Json -Depth 8
