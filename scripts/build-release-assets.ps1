param(
  [string]$Version = "0.0.1-ylem",
  [string]$Target = "windows-x64"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$dist = Join-Path $root "dist"
$stage = Join-Path $dist "qorx-v$Version-$Target"
$asset = Join-Path $dist "qorx-v$Version-$Target.zip"

Push-Location $root
try {
  cargo build --release --locked
  Get-ChildItem -LiteralPath target\release -File -Filter "*.pdb" -ErrorAction SilentlyContinue | Remove-Item -Force
  if (Test-Path $stage) {
    Remove-Item -LiteralPath $stage -Recurse -Force
  }
  New-Item -ItemType Directory -Path $stage | Out-Null
  Copy-Item target\release\qorx.exe $stage\
  Copy-Item README.md $stage\
  Copy-Item LICENSE $stage\
  Copy-Item NOTICE $stage\
  New-Item -ItemType Directory -Path (Join-Path $stage "docs") | Out-Null
  Copy-Item docs\COMMANDS.md -Destination (Join-Path $stage "docs")
  Copy-Item docs\INSTALL.md -Destination (Join-Path $stage "docs")
  Copy-Item docs\DISTRIBUTION.md -Destination (Join-Path $stage "docs")
  Copy-Item docs\PRODUCTION.md -Destination (Join-Path $stage "docs")
  Copy-Item docs\SERVER.md -Destination (Join-Path $stage "docs")
  Copy-Item docs\handbook (Join-Path $stage "docs\handbook") -Recurse
  Copy-Item docs\releases (Join-Path $stage "docs\releases") -Recurse
  $debugArtifacts = Get-ChildItem -LiteralPath $stage -Recurse -File | Where-Object { $_.Name -match '\.(pdb|map)$' }
  if ($debugArtifacts) {
    throw "Debug artifacts must not be included in release package: $($debugArtifacts.FullName -join ', ')"
  }

  if (Test-Path $asset) {
    Remove-Item -LiteralPath $asset -Force
  }
  Compress-Archive -Path (Join-Path $stage "*") -DestinationPath $asset -Force
  $hash = (Get-FileHash -Algorithm SHA256 $asset).Hash.ToLowerInvariant()
  Set-Content -Path "$asset.sha256" -Value "$hash  $(Split-Path -Leaf $asset)`n" -NoNewline
  [pscustomobject]@{
    asset = $asset
    sha256 = $hash
  } | ConvertTo-Json
}
finally {
  Pop-Location
}
