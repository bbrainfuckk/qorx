param(
    [string]$RepoRoot = ""
)

$ErrorActionPreference = "Stop"

if (-not $RepoRoot) {
    $RepoRoot = Split-Path -Parent $PSScriptRoot
}
$RepoRoot = (Resolve-Path -LiteralPath $RepoRoot).Path

$failures = New-Object System.Collections.Generic.List[string]

function Add-Failure {
    param([string]$Message)
    $failures.Add($Message) | Out-Null
}

function Repo-Path {
    param([string]$RelativePath)
    return Join-Path $RepoRoot $RelativePath
}

function Read-RepoText {
    param([string]$RelativePath)
    $path = Repo-Path $RelativePath
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        Add-Failure "missing $RelativePath"
        return ""
    }
    return Get-Content -LiteralPath $path -Raw
}

function Require-File {
    param([string]$RelativePath)
    if (-not (Test-Path -LiteralPath (Repo-Path $RelativePath) -PathType Leaf)) {
        Add-Failure "missing $RelativePath"
    }
}

function Require-Text {
    param(
        [string]$Name,
        [string]$Text,
        [string]$Pattern,
        [string]$Message
    )
    if ($Text -notmatch $Pattern) {
        Add-Failure "${Name}: $Message"
    }
}

function Reject-Text {
    param(
        [string]$Name,
        [string]$Text,
        [string]$Pattern,
        [string]$Message
    )
    if ($Text -match $Pattern) {
        Add-Failure "${Name}: $Message"
    }
}

$cargo = Read-RepoText "Cargo.toml"
$cargoVersionMatch = [regex]::Match($cargo, '(?m)^\s*version\s*=\s*"([^"]+)"')
if (-not $cargoVersionMatch.Success) {
    Add-Failure "Cargo.toml must expose package version"
    $cargoVersion = ""
} else {
    $cargoVersion = $cargoVersionMatch.Groups[1].Value
}

$requiredFiles = @(
    "packaging\README.md",
    "packaging\npm\package.json",
    "packaging\npm\bin\qorx.js",
    "packaging\npm\scripts\install.js",
    "packaging\pypi\pyproject.toml",
    "packaging\pypi\qorx_cli\launcher.py",
    "packaging\arch\PKGBUILD",
    "packaging\homebrew\qorx.rb",
    "packaging\scoop\qorx.json",
    "packaging\winget\Qorx.Qorx.yaml",
    "packaging\winget\Qorx.Qorx.installer.yaml",
    "packaging\winget\Qorx.Qorx.locale.en-US.yaml",
    "packaging\snap\snapcraft.yaml",
    "packaging\nfpm\qorx.yaml",
    "Dockerfile",
    "flake.nix",
    ".github\workflows\package-channels.yml"
)
foreach ($file in $requiredFiles) {
    Require-File $file
}

$packagingReadme = Read-RepoText "packaging\README.md"
$distribution = Read-RepoText "docs\DISTRIBUTION.md"
$install = Read-RepoText "docs\INSTALL.md"
$community = Read-RepoText "docs\COMMUNITY.md"
$readme = Read-RepoText "README.md"
$workflow = Read-RepoText ".github\workflows\package-channels.yml"
$releaseWorkflow = Read-RepoText ".github\workflows\release-platforms.yml"

foreach ($doc in @(
    @{ name = "packaging README"; text = $packagingReadme },
    @{ name = "distribution"; text = $distribution },
    @{ name = "install"; text = $install },
    @{ name = "community"; text = $community },
    @{ name = "README"; text = $readme }
)) {
    Require-Text $doc.name $doc.text 'PyPI' "must mention PyPI"
    Require-Text $doc.name $doc.text 'npm' "must mention npm"
    Require-Text $doc.name $doc.text 'Arch|AUR' "must mention Arch/AUR"
    Require-Text $doc.name $doc.text 'Homebrew' "must mention Homebrew"
    Require-Text $doc.name $doc.text 'WinGet|Scoop' "must mention Windows package managers"
    Require-Text $doc.name $doc.text 'Docker' "must mention Docker"
    Require-Text $doc.name $doc.text 'Nix' "must mention Nix"
    Require-Text $doc.name $doc.text '5,000 included Edge/Cloud requests' "must keep Edge Starter allowance visible"
    Reject-Text $doc.name $doc.text '(?i)Community Edition.*(stop|stops|expire|expires).*5,000' "must not claim CE stops at 5,000"
}

Require-Text "package workflow" $workflow 'check-package-channels\.ps1' "must run package-channel verification"
Require-Text "package workflow" $workflow 'node\s+-e' "must validate npm metadata"
Require-Text "package workflow" $workflow 'tomllib' "must validate PyPI metadata"
Require-Text "package workflow" $workflow 'docker build' "must validate Dockerfile"
Require-Text "release workflow" $releaseWorkflow 'macos-15-intel' "must use current Intel macOS runner label"

try {
    $npm = Get-Content -LiteralPath (Repo-Path "packaging\npm\package.json") -Raw | ConvertFrom-Json
    if ($npm.version -ne $cargoVersion) {
        Add-Failure "npm package version '$($npm.version)' does not match Cargo version '$cargoVersion'"
    }
    if (-not $npm.bin.qorx) {
        Add-Failure "npm package must expose qorx bin"
    }
} catch {
    Add-Failure "packaging/npm/package.json is not valid JSON"
}

try {
    $scoop = Get-Content -LiteralPath (Repo-Path "packaging\scoop\qorx.json") -Raw | ConvertFrom-Json
    if ($scoop.version -ne $cargoVersion) {
        Add-Failure "Scoop manifest version '$($scoop.version)' does not match Cargo version '$cargoVersion'"
    }
    if (-not $scoop.bin) {
        Add-Failure "Scoop manifest must expose bin"
    }
    if ($scoop.hash -match 'TO_BE_FILLED_AFTER_RELEASE') {
        Add-Failure "Scoop manifest must include the release asset SHA256"
    }
} catch {
    Add-Failure "packaging/scoop/qorx.json is not valid JSON"
}

$pyproject = Read-RepoText "packaging\pypi\pyproject.toml"
Require-Text "PyPI pyproject" $pyproject ('version\s*=\s*"' + [regex]::Escape($cargoVersion) + '"') "must match Cargo version"
Require-Text "PyPI pyproject" $pyproject 'qorx\s*=\s*"qorx_cli\.launcher:main"' "must expose qorx console script"

$pkgbuild = Read-RepoText "packaging\arch\PKGBUILD"
Require-Text "Arch PKGBUILD" $pkgbuild ('pkgver=' + [regex]::Escape($cargoVersion)) "must match Cargo version"
Require-Text "Arch PKGBUILD" $pkgbuild 'cargo build --release --locked' "must source-build locked Rust"

$homebrew = Read-RepoText "packaging\homebrew\qorx.rb"
Require-Text "Homebrew formula" $homebrew ('version "' + [regex]::Escape($cargoVersion) + '"') "must match Cargo version"
Require-Text "Homebrew formula" $homebrew 'cargo", "install"' "must source-build through cargo"

$snap = Read-RepoText "packaging\snap\snapcraft.yaml"
Require-Text "Snap manifest" $snap ('version:\s*"' + [regex]::Escape($cargoVersion) + '"') "must match Cargo version"
Require-Text "Snap manifest" $snap 'plugin:\s*rust' "must use rust plugin"

$dockerfile = Read-RepoText "Dockerfile"
Require-Text "Dockerfile" $dockerfile 'cargo build --release --locked' "must build locked release binary"

$flake = Read-RepoText "flake.nix"
Require-Text "Nix flake" $flake ('version = "' + [regex]::Escape($cargoVersion) + '"') "must match Cargo version"

$nfpm = Read-RepoText "packaging\nfpm\qorx.yaml"
Require-Text "nfpm config" $nfpm ('version:\s*' + [regex]::Escape($cargoVersion)) "must match Cargo version"

$wingetInstaller = Read-RepoText "packaging\winget\Qorx.Qorx.installer.yaml"
Reject-Text "WinGet installer manifest" $wingetInstaller 'TO_BE_FILLED_AFTER_RELEASE' "must include the release asset SHA256"

if ($failures.Count -gt 0) {
    [pscustomobject]@{
        ok = $false
        check = "package-channels"
        failures = $failures
    } | ConvertTo-Json -Depth 4
    exit 1
}

[pscustomobject]@{
    ok = $true
    check = "package-channels"
    version = $cargoVersion
    channels = @("PyPI", "npm", "Arch/AUR", "Homebrew", "Scoop", "WinGet", "Snap", "Docker", "Nix", "Deb/RPM via nfpm")
} | ConvertTo-Json -Depth 4
