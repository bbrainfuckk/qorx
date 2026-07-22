param([string]$RepoRoot = "")

$ErrorActionPreference = "Stop"
if (-not $RepoRoot) { $RepoRoot = Split-Path -Parent $PSScriptRoot }
$RepoRoot = (Resolve-Path -LiteralPath $RepoRoot).Path
$manifestPath = Join-Path $RepoRoot "compiler\bootstrap-manifest.json"
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
$cargo = Get-Content -LiteralPath (Join-Path $RepoRoot "Cargo.toml") -Raw
$version = [regex]::Match($cargo, '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value

if ($manifest.release -ne $version) {
  throw "bootstrap manifest release '$($manifest.release)' does not match Cargo version '$version'"
}

$compilerSource = Join-Path $RepoRoot $manifest.self_host_gate.compiler_source
$stage1 = Join-Path $RepoRoot $manifest.self_host_gate.stage1_bytecode
$stage2 = Join-Path $RepoRoot $manifest.self_host_gate.stage2_bytecode

if ($manifest.compiler.self_hosted) {
  foreach ($path in @($compilerSource, $stage1, $stage2)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
      throw "self-hosted=true requires $path"
    }
  }
  if ([IO.Path]::GetExtension($compilerSource) -ne ".qorx") {
    throw "the self-hosted compiler source must be a .qorx program"
  }
  $stage1Hash = (Get-FileHash -LiteralPath $stage1 -Algorithm SHA256).Hash
  $stage2Hash = (Get-FileHash -LiteralPath $stage2 -Algorithm SHA256).Hash
  if ($stage1Hash -ne $stage2Hash) {
    throw "stage-1 and stage-2 compiler bytecode are not reproducible"
  }
}

[ordered]@{
  ok = $true
  release = $version
  implementation = $manifest.compiler.implementation
  self_hosted = [bool]$manifest.compiler.self_hosted
  status = $manifest.self_host_gate.status
} | ConvertTo-Json
