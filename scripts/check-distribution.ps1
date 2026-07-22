param(
  [string]$Package = "qorx"
)

$ErrorActionPreference = "Continue"

Write-Host "GitHub"
git ls-remote --tags https://github.com/bbrainfuckk/qorx.git refs/tags/v1.0.5

Write-Host "crates.io"
try { Invoke-RestMethod "https://crates.io/api/v1/crates/$Package" | ConvertTo-Json -Depth 4 } catch { "missing" }

Write-Host "npm"
npm view $Package name version dist.tarball --json

Write-Host "PyPI"
try { Invoke-RestMethod "https://pypi.org/pypi/$Package/json" | ConvertTo-Json -Depth 4 } catch { "missing" }
