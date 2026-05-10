param(
  [string]$Version = "0.0.1-ylem",
  [switch]$DryRun
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot

function Require-Env($Name) {
  if (-not [Environment]::GetEnvironmentVariable($Name)) {
    throw "Missing environment variable: $Name"
  }
}

Push-Location $root
try {
  cargo package --allow-dirty
  if ($DryRun) {
    cargo publish --dry-run --allow-dirty
  } else {
    Require-Env "CARGO_REGISTRY_TOKEN"
    cargo publish --allow-dirty
  }

  Push-Location (Join-Path $root "packages\npm")
  try {
    npm pack --dry-run
    if (-not $DryRun) {
      if (-not ([Environment]::GetEnvironmentVariable("NPM_TOKEN") -or [Environment]::GetEnvironmentVariable("NODE_AUTH_TOKEN"))) {
        throw "Missing NPM_TOKEN or NODE_AUTH_TOKEN"
      }
      npm publish --access public
    }
  }
  finally {
    Pop-Location
  }

  Push-Location (Join-Path $root "packages\python")
  try {
    python -m pip install --upgrade build twine
    if (Test-Path dist) {
      Remove-Item -LiteralPath dist -Recurse -Force
    }
    python -m build
    $distFiles = Get-ChildItem -LiteralPath dist -File | Where-Object {
      $_.Name -like "qorx-$Version*"
    } | ForEach-Object { $_.FullName }
    if (-not $distFiles -or $distFiles.Count -eq 0) {
      throw "No Python dist files found for version $Version"
    }
    if ($DryRun) {
      python -m twine check @distFiles
    } else {
      Require-Env "TWINE_PASSWORD"
      if (-not [Environment]::GetEnvironmentVariable("TWINE_USERNAME")) {
        $env:TWINE_USERNAME = "__token__"
      }
      python -m twine upload @distFiles
    }
  }
  finally {
    Pop-Location
  }
}
finally {
  Pop-Location
}
