param(
    [string]$Compiler = "ng_selfhost_clean.exe",
    [string]$CacheRoot = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Resolve-WorkspacePath {
    param(
        [Parameter(Mandatory = $true)][string]$Workspace,
        [Parameter(Mandatory = $true)][string]$Path
    )

    if ([System.IO.Path]::IsPathRooted($Path)) {
        return [System.IO.Path]::GetFullPath($Path)
    }

    return [System.IO.Path]::GetFullPath((Join-Path $Workspace $Path))
}

$workspace = (Get-Location).Path
if ([string]::IsNullOrWhiteSpace($CacheRoot)) {
    $CacheRoot = Join-Path $env:LOCALAPPDATA "Limbaj\trust-root-cache"
}

$compilerPath = Resolve-WorkspacePath -Workspace $workspace -Path $Compiler
if (-not (Test-Path -LiteralPath $compilerPath)) {
    throw "missing trust root to back up: $compilerPath"
}

$compilerInfo = Get-Item -LiteralPath $compilerPath
$hash = (Get-FileHash -LiteralPath $compilerPath -Algorithm SHA256).Hash.ToUpperInvariant()
$cacheDir = Join-Path $CacheRoot $hash
$cachedExe = Join-Path $cacheDir "ng_selfhost_clean.exe"
$manifestPath = Join-Path $cacheDir "manifest.txt"
$latestPath = Join-Path $CacheRoot "latest.txt"

New-Item -ItemType Directory -Force -Path $cacheDir | Out-Null
Copy-Item -LiteralPath $compilerPath -Destination $cachedExe -Force

$manifest = @(
    "sha256=$hash"
    "size=$($compilerInfo.Length)"
    "cached_at=$((Get-Date).ToString('s'))"
    "source=$compilerPath"
)
Set-Content -LiteralPath $manifestPath -Value $manifest
Set-Content -LiteralPath $latestPath -Value $hash

Write-Output "TRUST ROOT BACKUP PASS"
Write-Output "source: $compilerPath"
Write-Output "cache:  $cachedExe"
Write-Output "size:   $($compilerInfo.Length)"
Write-Output "sha256: $hash"
