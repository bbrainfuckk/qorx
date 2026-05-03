param(
    [Parameter(Mandatory = $true)]
    [string]$Binary,

    [int64]$MaxBytes = 5242880
)

$ErrorActionPreference = "Stop"

$path = (Resolve-Path -LiteralPath $Binary).Path
$item = Get-Item -LiteralPath $path

$result = [pscustomobject]@{
    ok = ($item.Length -le $MaxBytes)
    check = "release-size"
    binary = $path
    bytes = $item.Length
    max_bytes = $MaxBytes
    mib = [math]::Round($item.Length / 1MB, 3)
    max_mib = [math]::Round($MaxBytes / 1MB, 3)
}

$result | ConvertTo-Json -Depth 3

if (-not $result.ok) {
    throw "release binary exceeds size cap"
}
