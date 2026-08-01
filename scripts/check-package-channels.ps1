param([string]$RepoRoot = "")

$ErrorActionPreference = "Stop"
if (-not $RepoRoot) { $RepoRoot = Split-Path -Parent $PSScriptRoot }
$RepoRoot = (Resolve-Path -LiteralPath $RepoRoot).Path
$failures = [Collections.Generic.List[string]]::new()
$warnings = [Collections.Generic.List[string]]::new()

function Text([string]$Relative) {
  $path = Join-Path $RepoRoot $Relative
  if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
    $failures.Add("missing $Relative")
    return ""
  }
  Get-Content -LiteralPath $path -Raw
}

function Require([string]$Name, [string]$Value, [string]$Pattern, [string]$Message) {
  if ($Value -notmatch $Pattern) { $failures.Add("${Name}: $Message") }
}

function Reject([string]$Name, [string]$Value, [string]$Pattern, [string]$Message) {
  if ($Value -match $Pattern) { $failures.Add("${Name}: $Message") }
}

$cargo = Text "Cargo.toml"
$match = [regex]::Match($cargo, '(?m)^version\s*=\s*"([^"]+)"')
if (-not $match.Success) { $failures.Add("Cargo.toml: missing package version") }
$version = $match.Groups[1].Value
$tag = "v$version"

foreach ($relative in @(
  "package.json",
  "packages\npm\package.json",
  "packages\python\pyproject.toml",
  "packages\python\src\qorx\runner.py",
  "packaging\arch\PKGBUILD",
  "packaging\aur\PKGBUILD",
  "packaging\homebrew\qorx.rb",
  "packaging\scoop\qorx.json",
  "packaging\winget\Qorx.Qorx.installer.yaml",
  "packaging\snap\snapcraft.yaml",
  "packaging\nfpm\qorx.yaml",
  "Dockerfile",
  "docker-compose.yml",
  "flake.nix",
  ".github\workflows\release-assets.yml",
  ".github\workflows\publish-registries.yml"
)) { [void](Text $relative) }

foreach ($relative in @("package.json", "packages\npm\package.json")) {
  try {
    $json = Text $relative | ConvertFrom-Json
    if ($json.version -ne $version) { $failures.Add("${relative}: version must be $version") }
    if (-not $json.bin.qorx) { $failures.Add("${relative}: missing qorx executable") }
  } catch { $failures.Add("${relative}: invalid JSON") }
}

$python = Text "packages\python\pyproject.toml"
Require "PyPI" $python ('version\s*=\s*"' + [regex]::Escape($version) + '"') "version must be $version"
Require "PyPI" $python 'qorx\s*=\s*"qorx\.runner:main"' "missing qorx entry point"
Require "Python runner" (Text "packages\python\src\qorx\runner.py") ('VERSION\s*=\s*"' + [regex]::Escape($version) + '"') "version must be $version"

$release = Text ".github\workflows\release-assets.yml"
foreach ($target in @("windows-x64", "windows-arm64", "linux-x64", "linux-arm64", "macos-x64", "macos-arm64")) {
  Require "release assets" $release ([regex]::Escape("name: $target")) "missing $target"
}
Require "release assets" $release 'qorx-\$\{tag\}-\$\{\{ matrix\.name \}\}' "asset names must be tag and platform specific"

$arch = Text "packaging\arch\PKGBUILD"
Require "Arch" $arch ('_cratever=' + [regex]::Escape($version)) "crate version must be $version"
Require "Arch" $arch 'arch=\("x86_64" "aarch64"\)' "must support x86_64 and aarch64"
$aur = Text "packaging\aur\PKGBUILD"
Require "AUR" $aur ('pkgver=' + [regex]::Escape($version)) "version must be $version"
Require "AUR" $aur 'arch=\(''x86_64'' ''aarch64''\)' "must support x86_64 and aarch64"
Require "AUR" $aur 'crates\.io/api/v1/crates/\$\{pkgname\}/\$\{pkgver\}/download' "must build the published crate"
Reject "AUR" $aur 'docs/COMMANDS\.md' "must not install missing files"
Require "Homebrew" (Text "packaging\homebrew\qorx.rb") ('tag:\s+"' + [regex]::Escape($tag) + '"') "tag must be $tag"
Require "Snap" (Text "packaging\snap\snapcraft.yaml") ('version:\s*"' + [regex]::Escape($version) + '"') "version must be $version"
Require "Nix" (Text "flake.nix") ('version = "' + [regex]::Escape($version) + '"') "version must be $version"
Require "nfpm" (Text "packaging\nfpm\qorx.yaml") ('version:\s*' + [regex]::Escape($version)) "version must be $version"

$dockerfile = Text "Dockerfile"
$compose = Text "docker-compose.yml"
Require "Dockerfile" $dockerfile ('ARG QORX_VERSION=' + [regex]::Escape($version)) "build version must be $version"
Require "Dockerfile" $dockerfile 'qorx --version.*qorx \$\{QORX_VERSION\}' "build must assert the runtime version"
Require "Dockerfile" $dockerfile 'USER qorx' "runtime must use the non-root qorx user"
Require "Compose" $compose ('QORX_VERSION:\s*"' + [regex]::Escape($version) + '"') "build version must be $version"
Require "Compose" $compose '127\.0\.0\.1:47187:47187' "host port must stay loopback-only"

$scoop = Text "packaging\scoop\qorx.json"
$winget = Text "packaging\winget\Qorx.Qorx.installer.yaml"
Require "Scoop" $scoop ([regex]::Escape("/$tag/qorx-$tag-windows-x64.zip")) "release URL must target $tag"
Require "WinGet" $winget ([regex]::Escape("/$tag/qorx-$tag-windows-x64.zip")) "release URL must target $tag"
if (($scoop + $winget) -match 'PENDING_|REPLACE_|TO_BE_FILLED') {
  $warnings.Add("Scoop/WinGet hashes remain pending until the v$version Windows asset is built")
}

foreach ($relative in @("packaging\README.md", "packaging\npm\README.md", "packaging\pypi\README.md", ".github\workflows\release-assets.yml")) {
  Reject $relative (Text $relative) '(?i)Community Edition|OSS Edition|Qorx Core' "use the Qorx product name"
}

$publish = Text ".github\workflows\publish-registries.yml"
Require "registry workflow" $publish 'CARGO_REGISTRY_TOKEN' "missing crates.io publishing"
Require "registry workflow" $publish '(?s)npm:\s+name: npm.*?environment:\s*npm.*?id-token:\s*write.*?node-version:\s*"24".*?npm@11\.5\.1' "missing npm trusted publishing"
Require "registry workflow" $publish 'id-token:\s*write' "missing PyPI trusted publishing"

$result = [ordered]@{
  ok = $failures.Count -eq 0
  check = "package-channels"
  version = $version
  tag = $tag
  release_targets = @("windows-x64", "windows-arm64", "linux-x64", "linux-arm64", "macos-x64", "macos-arm64")
  registry_publish_ready = $warnings.Count -eq 0
  warnings = @($warnings)
  failures = @($failures)
}
$result | ConvertTo-Json -Depth 5
if ($failures.Count -gt 0) { exit 1 }
