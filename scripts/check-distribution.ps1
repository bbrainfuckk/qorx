param(
  [string]$Package = "qorx"
)

$ErrorActionPreference = "Continue"

Write-Host "GitHub"
gh release view v0.0.1-ylem --repo bbrainfuckk/qorx --json tagName,isDraft,isPrerelease,url,assets

Write-Host "crates.io"
try { Invoke-RestMethod "https://crates.io/api/v1/crates/$Package" | ConvertTo-Json -Depth 4 } catch { "missing" }

Write-Host "npm"
npm view $Package name version dist.tarball --json

Write-Host "PyPI"
try { Invoke-RestMethod "https://pypi.org/pypi/$Package/json" | ConvertTo-Json -Depth 4 } catch { "missing" }
