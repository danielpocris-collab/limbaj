param(
    [string]$Destination = "ng_selfhost_clean.exe",
    [string]$CacheRoot = "",
    [string]$Hash = "",
    [switch]$Force
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

if (-not (Test-Path -LiteralPath $CacheRoot)) {
    throw "missing trust root cache: $CacheRoot"
}

if ([string]::IsNullOrWhiteSpace($Hash)) {
    $latestPath = Join-Path $CacheRoot "latest.txt"
    if (Test-Path -LiteralPath $latestPath) {
        $Hash = (Get-Content -LiteralPath $latestPath -Raw).Trim()
    }
}

if ([string]::IsNullOrWhiteSpace($Hash)) {
    $cachedDirs = Get-ChildItem -LiteralPath $CacheRoot -Directory | Sort-Object LastWriteTime -Descending
    if ($cachedDirs.Count -eq 0) {
        throw "trust root cache is empty: $CacheRoot"
    }
    $Hash = $cachedDirs[0].Name
}

$cacheDir = Join-Path $CacheRoot $Hash
$cachedExe = Join-Path $cacheDir "ng_selfhost_clean.exe"
if (-not (Test-Path -LiteralPath $cachedExe)) {
    throw "missing cached trust root: $cachedExe"
}

$destinationPath = Resolve-WorkspacePath -Workspace $workspace -Path $Destination
if ((Test-Path -LiteralPath $destinationPath) -and (-not $Force.IsPresent)) {
    throw "destination already exists, use -Force to overwrite: $destinationPath"
}

Copy-Item -LiteralPath $cachedExe -Destination $destinationPath -Force

$restoredHash = (Get-FileHash -LiteralPath $destinationPath -Algorithm SHA256).Hash.ToUpperInvariant()
if ($restoredHash -ne $Hash.ToUpperInvariant()) {
    throw "restored trust root hash mismatch: expected $Hash got $restoredHash"
}

$restoredInfo = Get-Item -LiteralPath $destinationPath

Write-Output "TRUST ROOT RESTORE PASS"
Write-Output "source: $cachedExe"
Write-Output "dest:   $destinationPath"
Write-Output "size:   $($restoredInfo.Length)"
Write-Output "sha256: $restoredHash"
