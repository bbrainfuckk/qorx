param(
  [Parameter(Mandatory = $true)]
  [ValidatePattern('^\d+\.\d+\.\d+([-.][0-9A-Za-z.-]+)?$')]
  [string]$Version,
  [string]$RepoRoot = "",
  [string]$CrateSha256 = "",
  [string]$WindowsZipSha256 = "",
  [string]$HomebrewRevision = "",
  [switch]$NoVerify,
  [switch]$NoBuild,
  [switch]$DryRunRegistries
)

$ErrorActionPreference = "Stop"
if (-not $RepoRoot) {
  $scriptRoot = if ($PSScriptRoot) { $PSScriptRoot } else { Split-Path -Parent $MyInvocation.MyCommand.Path }
  $RepoRoot = Split-Path -Parent $scriptRoot
}
$RepoRoot = (Resolve-Path -LiteralPath $RepoRoot).Path
$Tag = "v$Version"
$Today = (Get-Date).ToUniversalTime()
$ReleaseDate = $Today.ToString("yyyy-MM-dd", [Globalization.CultureInfo]::InvariantCulture)
$DebianDate = $Today.ToString("ddd, dd MMM yyyy HH:mm:ss +0000", [Globalization.CultureInfo]::InvariantCulture)
$script:Changed = [System.Collections.Generic.List[string]]::new()
$script:Warnings = [System.Collections.Generic.List[string]]::new()

function Join-Repo([string]$Relative) {
  Join-Path $RepoRoot $Relative
}

function Read-RepoText([string]$Relative) {
  $path = Join-Repo $Relative
  if (-not (Test-Path -LiteralPath $path)) {
    throw "Missing file: $Relative"
  }
  [IO.File]::ReadAllText($path)
}

function Write-RepoText([string]$Relative, [string]$Text) {
  $path = Join-Repo $Relative
  $old = if (Test-Path -LiteralPath $path) { [IO.File]::ReadAllText($path) } else { "" }
  if ($old -ne $Text) {
    $dir = Split-Path -Parent $path
    if ($dir) {
      [IO.Directory]::CreateDirectory($dir) | Out-Null
    }
    $utf8 = [Text.UTF8Encoding]::new($false)
    [IO.File]::WriteAllText($path, $Text, $utf8)
    $script:Changed.Add($Relative)
  }
}

function Replace-RepoText([string]$Relative, [string]$Pattern, [scriptblock]$Replace, [switch]$Optional) {
  $text = Read-RepoText $Relative
  $regex = [Text.RegularExpressions.Regex]::new($Pattern)
  $matchCount = $regex.Matches($text).Count
  $callback = [Text.RegularExpressions.MatchEvaluator]{ param($match) & $Replace $match }
  $next = $regex.Replace($text, $callback)
  if ($matchCount -eq 0 -and -not $Optional) {
    $script:Warnings.Add("No match in $Relative for $Pattern")
  }
  Write-RepoText $Relative $next
}

function Warn([string]$Message) {
  $script:Warnings.Add($Message)
  Write-Warning $Message
}

function Run([string]$File, [string[]]$Args, [string]$Cwd = $RepoRoot) {
  Push-Location $Cwd
  try {
    Write-Host ("+ {0} {1}" -f $File, ($Args -join " "))
    & $File @Args
    if ($LASTEXITCODE -ne 0) {
      throw "Command failed with exit ${LASTEXITCODE}: $File $($Args -join ' ')"
    }
  }
  finally {
    Pop-Location
  }
}

function Current-CargoVersion {
  $text = Read-RepoText "Cargo.toml"
  $match = [regex]::Match($text, '(?m)^version\s*=\s*"([^"]+)"')
  if (-not $match.Success) {
    throw "Cargo.toml package version not found"
  }
  $match.Groups[1].Value
}

function Set-TomlVersion([string]$Relative) {
  Replace-RepoText $Relative '(?m)^(version\s*=\s*)"[^"]+"' {
    param($m) $m.Groups[1].Value + '"' + $Version + '"'
  }
}

function Set-JsonVersion([string]$Relative) {
  Replace-RepoText $Relative '("version"\s*:\s*)"[^"]+"' {
    param($m) $m.Groups[1].Value + '"' + $Version + '"'
  }
}

function Set-PythonInitVersion([string]$Relative) {
  Replace-RepoText $Relative '(?m)^(__version__\s*=\s*)"[^"]+"' {
    param($m) $m.Groups[1].Value + '"' + $Version + '"'
  }
}

function Set-PythonRunnerVersion([string]$Relative) {
  Replace-RepoText $Relative '(?m)^(VERSION\s*=\s*)"[^"]+"' {
    param($m) $m.Groups[1].Value + '"' + $Version + '"'
  }
}

function Set-CargoLockVersion {
  Replace-RepoText "Cargo.lock" '(?ms)(\[\[package\]\]\r?\nname = "qorx"\r?\nversion = ")[^"]+(")' {
    param($m) $m.Groups[1].Value + $Version + $m.Groups[2].Value
  }
}

function Set-AurFiles([string]$PreviousVersion) {
  $pkgbuild = "packaging\aur\PKGBUILD"
  $source = '${pkgname}-${pkgver}.tar.gz::https://crates.io/api/v1/crates/${pkgname}/${pkgver}/download'
  Replace-RepoText $pkgbuild '(?m)^(pkgver=).*$' { param($m) $m.Groups[1].Value + $Version }
  Replace-RepoText $pkgbuild "(?m)^source=.*$" { param($m) 'source=("' + $source + '")' }

  $currentHashMatch = [regex]::Match((Read-RepoText $pkgbuild), "(?m)^sha256sums=\('([^']+)'\)")
  $hash = if ($CrateSha256) {
    $CrateSha256.ToLowerInvariant()
  } elseif ($Version -eq $PreviousVersion -and $currentHashMatch.Success) {
    $currentHashMatch.Groups[1].Value
  } else {
    Warn "AUR crate sha256 is unknown for $Version; PKGBUILD/.SRCINFO use SKIP until makepkg -g or crates.io hash is filled."
    "SKIP"
  }
  Replace-RepoText $pkgbuild "(?m)^sha256sums=.*$" { param($m) "sha256sums=('$hash')" }

  $srcinfo = @"
pkgbase = qorx
	pkgdesc = Qorx AI-native language and runtime for local context resolution
	pkgver = $Version
	pkgrel = 1
	url = https://github.com/bbrainfuckk/qorx
	arch = x86_64
	arch = aarch64
	license = AGPL-3.0-only
	makedepends = rust
	source = qorx-$Version.tar.gz::https://crates.io/api/v1/crates/qorx/$Version/download
	sha256sums = $hash

pkgname = qorx
"@
  Write-RepoText "packaging\aur\.SRCINFO" ($srcinfo + "`n")
}

function Set-RpmDebSnap([string]$PreviousVersion) {
  Replace-RepoText "packaging\rpm\qorx.spec" '(?m)^(Version:\s*)\S+' {
    param($m) $m.Groups[1].Value + $Version
  }
  if ($Version -ne $PreviousVersion) {
    Replace-RepoText "packaging\rpm\qorx.spec" '(?m)^\* .+ - [0-9][^-]+-1$' {
      param($m) "* $($Today.ToString('ddd MMM dd yyyy', [Globalization.CultureInfo]::InvariantCulture)) Marvin Sarreal Villanueva <marvin@orin.work> - $Version-1"
    }
    $debian = @"
qorx ($Version-1) unstable; urgency=medium

  * Package Qorx distribution release.

 -- Marvin Sarreal Villanueva <marvin@orin.work>  $DebianDate
"@
    Write-RepoText "packaging\debian\changelog" ($debian + "`n")
  }
  Replace-RepoText "snap\snapcraft.yaml" '(?m)^(version:\s*)"[^\"]+"' {
    param($m) $m.Groups[1].Value + '"' + $Version + '"'
  }
}

function Resolve-WindowsZipSha([string]$PreviousVersion) {
  if ($WindowsZipSha256) {
    return $WindowsZipSha256.ToUpperInvariant()
  }
  $shaFile = Join-Repo "dist\qorx-v$Version-windows-x64.zip.sha256"
  if (Test-Path -LiteralPath $shaFile) {
    $first = (Get-Content -LiteralPath $shaFile -TotalCount 1).Split(" ")[0]
    if ($first) {
      return $first.ToUpperInvariant()
    }
  }
  $scoop = Read-RepoText "packaging\scoop\qorx.json"
  $match = [regex]::Match($scoop, '"hash"\s*:\s*"([^"]+)"')
  if ($Version -eq $PreviousVersion -and $match.Success) {
    return $match.Groups[1].Value.ToUpperInvariant()
  }
  Warn "Windows release asset sha256 is unknown for $Version; Scoop/WinGet use REPLACE_WITH_WINDOWS_ZIP_SHA256 until the zip is built."
  "REPLACE_WITH_WINDOWS_ZIP_SHA256"
}

function Set-DesktopPackageFiles([string]$PreviousVersion) {
  $zipHash = Resolve-WindowsZipSha $PreviousVersion
  $zipHashLower = $zipHash.ToLowerInvariant()

  Set-JsonVersion "packaging\scoop\qorx.json"
  Replace-RepoText "packaging\scoop\qorx.json" 'https://github\.com/bbrainfuckk/qorx/releases/download/v[0-9][^/]+/qorx-v[0-9][^/]+-windows-x64\.zip' {
    param($m) "https://github.com/bbrainfuckk/qorx/releases/download/$Tag/qorx-$Tag-windows-x64.zip"
  }
  Replace-RepoText "packaging\scoop\qorx.json" '("hash"\s*:\s*")[^"]+(")' {
    param($m) $m.Groups[1].Value + $zipHashLower + $m.Groups[2].Value
  }

  $wingetRoot = "packaging\winget\manifests\b\bbrainfuckk\qorx"
  $latestDir = Join-Repo (Join-Path $wingetRoot $PreviousVersion)
  $targetDirRel = Join-Path $wingetRoot $Version
  $targetDir = Join-Repo $targetDirRel
  if (-not (Test-Path -LiteralPath $targetDir)) {
    if (-not (Test-Path -LiteralPath $latestDir)) {
      throw "Cannot create WinGet manifest. Missing template: $latestDir"
    }
    Copy-Item -LiteralPath $latestDir -Destination $targetDir -Recurse
    $script:Changed.Add($targetDirRel)
  }
  foreach ($file in Get-ChildItem -LiteralPath $targetDir -File -Filter "*.yaml") {
    $rel = $file.FullName.Substring($RepoRoot.Length + 1)
    Replace-RepoText $rel '(?m)^(PackageVersion:\s*)\S+' { param($m) $m.Groups[1].Value + $Version }
    Replace-RepoText $rel 'https://github\.com/bbrainfuckk/qorx/releases/download/v[0-9][^/]+/qorx-v[0-9][^/]+-windows-x64\.zip' {
      param($m) "https://github.com/bbrainfuckk/qorx/releases/download/$Tag/qorx-$Tag-windows-x64.zip"
    } -Optional
    Replace-RepoText $rel '(?m)^(    InstallerSha256:\s*)\S+' { param($m) $m.Groups[1].Value + $zipHash } -Optional
  }
}

function Set-Homebrew([string]$PreviousVersion) {
  Replace-RepoText "packaging\homebrew\qorx.rb" 'tag:\s+"v[^"]+"' {
    param($m) 'tag:      "' + $Tag + '"'
  }
  if ($HomebrewRevision) {
    $text = Read-RepoText "packaging\homebrew\qorx.rb"
    if ($text -match 'revision:\s*"[^"]+"') {
      Replace-RepoText "packaging\homebrew\qorx.rb" 'revision:\s+"[^"]+"' {
        param($m) 'revision: "' + $HomebrewRevision + '"'
      }
    } else {
      Replace-RepoText "packaging\homebrew\qorx.rb" '(?m)^(\s*tag:\s+"[^"]+")' {
        param($m) $m.Groups[1].Value + ",`n      revision: `"$HomebrewRevision`""
      }
    }
  } else {
    Replace-RepoText "packaging\homebrew\qorx.rb" '(?m)^\s*revision:\s*"[^"]+",?\r?\n' {
      param($m) ""
    } -Optional
    Replace-RepoText "packaging\homebrew\qorx.rb" 'tag:\s+"([^"]+)",' {
      param($m) 'tag:      "' + $m.Groups[1].Value + '"'
    } -Optional
  }
  Replace-RepoText "packaging\homebrew\qorx.rb" 'qorx [0-9]+\.[0-9]+\.[0-9]+([-.][0-9A-Za-z.-]+)?' {
    param($m) "qorx $Version"
  }
}

function Set-DocsAndScriptDefaults([string]$PreviousVersion) {
  $reviewDoc = "docs\QORX_$(($Version -replace '\.', '_'))_RUST.md"
  $previousReviewDoc = "docs\QORX_$(($PreviousVersion -replace '\.', '_'))_RUST.md"
  if ($Version -ne $PreviousVersion -and
      (Test-Path -LiteralPath (Join-Repo $previousReviewDoc)) -and
      -not (Test-Path -LiteralPath (Join-Repo $reviewDoc))) {
    Move-Item -LiteralPath (Join-Repo $previousReviewDoc) -Destination (Join-Repo $reviewDoc)
  }

  foreach ($relative in @(
      "README.md",
      "docs\index.md",
      "docs\DISTRIBUTION.md",
      "docs\MEDIA.md",
      $reviewDoc,
      "docs\papers\qorx-local-context-resolution-preprint.md",
      "scripts\build-release-assets.ps1",
      "scripts\publish-registries.ps1",
      "scripts\check-distribution.ps1"
    )) {
    if (-not (Test-Path -LiteralPath (Join-Repo $relative))) {
      continue
    }
    Replace-RepoText $relative 'v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?' { param($m) $Tag } -Optional
    Replace-RepoText $relative '(?<![A-Za-z0-9_.-])([0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?)(?![A-Za-z0-9_.-])' { param($m) $Version } -Optional
  }
  $reviewDocLeaf = Split-Path -Leaf $reviewDoc
  foreach ($relative in @("README.md", "docs\index.md")) {
    Replace-RepoText $relative 'QORX_[0-9_]+_RUST\.md' { param($m) $reviewDocLeaf } -Optional
  }
  Replace-RepoText "CITATION.cff" '(?m)^(version:\s*)"[^"]+"' { param($m) $m.Groups[1].Value + '"' + $Version + '"' }
  if ($Version -ne $PreviousVersion) {
    Replace-RepoText "CITATION.cff" '(?m)^(date-released:\s*)"[0-9-]+"' { param($m) $m.Groups[1].Value + '"' + $ReleaseDate + '"' }
  }
  Set-JsonVersion ".zenodo.json"
}

function Set-InstallDoc {
  $doc = "docs\INSTALL.md"
  Replace-RepoText $doc '--tag v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?' {
    param($m) "--tag $Tag"
  } -Optional
  Replace-RepoText $doc 'releases/download/v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?/' {
    param($m) "releases/download/$Tag/"
  } -Optional
  Replace-RepoText $doc 'qorx-v[0-9]+\.[0-9]+\.[0-9]+' {
    param($m) "qorx-$Tag"
  } -Optional
  Replace-RepoText $doc 'qorx-npm-v[0-9]+\.[0-9]+\.[0-9]+' {
    param($m) "qorx-npm-$Tag"
  } -Optional
  Replace-RepoText $doc 'qorx-[0-9]+\.[0-9]+\.[0-9]+-py3-none-any\.whl' {
    param($m) "qorx-$Version-py3-none-any.whl"
  } -Optional
}

$previousVersion = Current-CargoVersion

Set-TomlVersion "Cargo.toml"
Set-CargoLockVersion
Set-JsonVersion "package.json"
Set-JsonVersion "packages\npm\package.json"
Set-TomlVersion "packages\python\pyproject.toml"
Set-PythonInitVersion "packages\python\src\qorx\__init__.py"
Set-PythonRunnerVersion "packages\python\src\qorx\runner.py"
Set-AurFiles $previousVersion
Set-RpmDebSnap $previousVersion
Set-DesktopPackageFiles $previousVersion
Set-Homebrew $previousVersion
Set-DocsAndScriptDefaults $previousVersion
Set-InstallDoc

$manifest = [ordered]@{
  schema = "qorx.release-prep.v1"
  version = $Version
  tag = $Tag
  generated_at = $Today.ToString("o", [Globalization.CultureInfo]::InvariantCulture)
  changed_files = @($script:Changed)
  warnings = @($script:Warnings)
  surfaces = @(
    "Cargo/crates.io",
    "root npm package",
    "packages/npm scoped package",
    "PyPI wrapper",
    "Arch/AUR",
    "Homebrew/Linuxbrew",
    "Debian",
    "RPM",
    "Snap",
    "Scoop",
    "WinGet",
    "Zenodo/CITATION metadata",
    "GitHub release docs"
  )
}

$manifestPath = "target\release\qorx-$Tag-release-prep.json"
Write-RepoText $manifestPath (($manifest | ConvertTo-Json -Depth 8) + "`n")

if (-not $NoVerify) {
  Run "cargo" @("fmt", "--check")
  Run "cargo" @("test")
  Run "cargo" @("clippy", "--all-targets", "--", "-D", "warnings")
  if (-not $NoBuild) {
    Run "cargo" @("build", "--release", "--locked")
    Run (Join-Repo "target\release\qorx.exe") @("--version")
    Run (Join-Repo "target\release\qorx.exe") @("doctor", "--json")
  }
  Run "npm" @("--version")
  Run "node" @("--version")
  Run "npm" @("pack", "--dry-run") (Join-Repo "packages\npm")
  Run "python" @("--version")
  if ($DryRunRegistries) {
    Run "powershell" @("-NoProfile", "-ExecutionPolicy", "Bypass", "-File", (Join-Repo "scripts\publish-registries.ps1"), "-Version", $Version, "-DryRun")
  }
}

if ($script:Warnings.Count -gt 0) {
  Write-Host "Release prep completed with warnings:"
  foreach ($warning in $script:Warnings) {
    Write-Host "- $warning"
  }
}

Write-Host "Release prep manifest: $manifestPath"
