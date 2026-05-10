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

function Read-RepoText {
    param([string]$RelativePath)
    $path = Join-Path $RepoRoot $RelativePath
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        Add-Failure "missing $RelativePath"
        return ""
    }
    return Get-Content -LiteralPath $path -Raw
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

$workflow = Read-RepoText ".github\workflows\testsprite-enterprise.yml"
Require-Text "workflow" $workflow '(?m)^\s*workflow_dispatch\s*:' "must be manually runnable"
Require-Text "workflow" $workflow 'TestSprite/run-action@v1' "must use the official TestSprite action"
Require-Text "workflow" $workflow 'testsprite-api-key:\s*\$\{\{\s*secrets\.TESTSPRITE_API_KEY\s*\}\}' "must read the API key from the GitHub secret"
Require-Text "workflow" $workflow 'base_url:\s*\$\{\{\s*inputs\.base_url\s*\}\}' "must use the operator-supplied public base URL"
Require-Text "workflow" $workflow 'github-token:\s*\$\{\{\s*github\.token\s*\}\}' "must pass the GitHub token expected by the TestSprite action"
Require-Text "workflow" $workflow 'blocking:\s*\$\{\{\s*inputs\.blocking\s*\}\}' "must make blocking mode explicit"
Require-Text "workflow" $workflow 'continue-on-error:\s*\$\{\{\s*inputs\.blocking\s*==\s*''false''\s*\}\}' "must let non-blocking cloud runs report without failing the workflow"
Require-Text "workflow" $workflow 'mkdir -p testsprite_tests/tmp' "must create the TestSprite output directory before the action runs"
Require-Text "workflow" $workflow 'cargo build --release --locked' "must build the checked-in release binary"
Require-Text "workflow" $workflow '127\.0\.0\.1:47187/health' "must run a local daemon health smoke before cloud testing"
Require-Text "workflow" $workflow 'TESTSPRITE_BASE_URL:\s*\$\{\{\s*inputs\.base_url\s*\}\}' "must expose the public base URL to repo-managed TestSprite tests"

$docs = Read-RepoText "docs\TESTSPRITE.md"
Require-Text "docs" $docs 'TESTSPRITE_API_KEY' "must document the GitHub secret name"
Require-Text "docs" $docs '(?i)revoke|rotate' "must tell operators to revoke or rotate leaked keys"
Require-Text "docs" $docs '(?i)public staging URL|public HTTPS URL|reachable.*URL' "must explain that TestSprite needs a reachable target"
Require-Text "docs" $docs 'TestSprite Enterprise QA' "must name the workflow"
Require-Text "docs" $docs '(?i)does not generate|only runs' "must not imply the GitHub Action generates the suite"
Require-Text "docs" $docs 'testsprite_tests/' "must document repo-managed TestSprite suite files"

$suiteJsonPath = Join-Path $RepoRoot "testsprite_tests\tmp\test_results.json"
if (-not (Test-Path -LiteralPath $suiteJsonPath -PathType Leaf)) {
    Add-Failure "missing testsprite_tests/tmp/test_results.json"
} else {
    $suiteRaw = Get-Content -LiteralPath $suiteJsonPath -Raw
    if ($suiteRaw -match 'C:\\Users\\|Traceback \(most recent call last\)') {
        Add-Failure "testsprite_tests/tmp/test_results.json must not contain local absolute paths or raw traceback output"
    }
    try {
        $suite = $suiteRaw | ConvertFrom-Json
        if (-not $suite -or $suite.Count -lt 1) {
            Add-Failure "testsprite_tests/tmp/test_results.json must contain at least one suite entry"
        }
        foreach ($case in @($suite)) {
            if (-not $case.title) {
                Add-Failure "TestSprite suite entry is missing title"
                continue
            }
            if (($case.PSObject.Properties.Name -contains "testStatus") -and ($case.testStatus -notin @("PASSED", "FAILED"))) {
                Add-Failure "TestSprite suite entry '$($case.title)' has invalid testStatus '$($case.testStatus)'"
            }
            if (($case.PSObject.Properties.Name -contains "testError") -and ($null -eq $case.testError)) {
                Add-Failure "TestSprite suite entry '$($case.title)' has null testError"
            }
            if ($case.testType -notin @("FRONTEND", "BACKEND")) {
                Add-Failure "TestSprite suite entry '$($case.title)' has invalid testType '$($case.testType)'"
            }
            $fileName = ($case.title -replace '\s+', '_') -replace '[^a-zA-Z0-9._-]', ''
            $caseFile = Join-Path $RepoRoot ("testsprite_tests\{0}.py" -f $fileName)
            if (-not (Test-Path -LiteralPath $caseFile -PathType Leaf) -and $case.testId) {
                $caseFile = Get-ChildItem -LiteralPath (Join-Path $RepoRoot "testsprite_tests") -File -Filter "$($case.testId)_*.py" |
                    Select-Object -First 1 -ExpandProperty FullName
            }
            if (-not (Test-Path -LiteralPath $caseFile -PathType Leaf)) {
                Add-Failure "missing TestSprite python file for suite title '$($case.title)'"
            } else {
                $caseText = Get-Content -LiteralPath $caseFile -Raw
                if ($caseText -match 'http://localhost:5173') {
                    Add-Failure "TestSprite python file '$($caseFile | Split-Path -Leaf)' hardcodes localhost instead of using TESTSPRITE_BASE_URL"
                }
                if ($caseText -notmatch 'TESTSPRITE_BASE_URL') {
                    Add-Failure "TestSprite python file '$($caseFile | Split-Path -Leaf)' does not read TESTSPRITE_BASE_URL"
                }
            }
        }
    } catch {
        Add-Failure "testsprite_tests/tmp/test_results.json is not valid JSON"
    }
}

$security = Read-RepoText "SECURITY.md"
Require-Text "SECURITY" $security 'TESTSPRITE_API_KEY' "must include the TestSprite key in the secret-handling policy"
Require-Text "SECURITY" $security '(?i)TestSprite.*secret|secret.*TestSprite' "must mention TestSprite secret handling"

$textExtensions = @(
    ".md", ".ps1", ".yml", ".yaml", ".toml", ".json", ".cff", ".rs", ".js", ".py",
    ".rb", ".spec", ".lock", ".txt", ".sh", ".nix"
)
$literalSecretPattern = '(?i)\bsk-(user|test|live)-[A-Za-z0-9_-]{24,}'
Get-ChildItem -LiteralPath $RepoRoot -Recurse -File | Where-Object {
    $full = $_.FullName
    $relative = $full.Substring($RepoRoot.Length).TrimStart('\', '/')
    $relative -notmatch '(^|[\\/])(\.git|target|dist|node_modules|\.venv|packages[\\/]python[\\/]dist)([\\/]|$)' -and
    $textExtensions -contains $_.Extension
} | ForEach-Object {
    $relative = $_.FullName.Substring($RepoRoot.Length).TrimStart('\', '/')
    $content = Get-Content -LiteralPath $_.FullName -Raw
    if ($content -match $literalSecretPattern) {
        Add-Failure "literal TestSprite-style secret found in $relative"
    }
}

if ($failures.Count -gt 0) {
    [pscustomobject]@{
        ok = $false
        gate = "testsprite-enterprise"
        failures = $failures
    } | ConvertTo-Json -Depth 4
    exit 1
}

[pscustomobject]@{
    ok = $true
    gate = "testsprite-enterprise"
    workflow = ".github/workflows/testsprite-enterprise.yml"
    docs = "docs/TESTSPRITE.md"
} | ConvertTo-Json -Depth 4
