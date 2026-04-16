param(
    [Parameter(Mandatory = $true)][string]$Workspace,
    [Parameter(Mandatory = $true)][string]$CompilerExe,
    [Parameter(Mandatory = $true)][string]$CaseJson
)

$ErrorActionPreference = "Stop"

$workspace = Resolve-Path -LiteralPath $Workspace
$caseDir = Split-Path -Parent $CaseJson

. (Join-Path $workspace "direct_toolchain_common.ps1")

$case = Get-Content -LiteralPath $CaseJson -Raw | ConvertFrom-Json
$result = Invoke-RuntimeCorpusCase -Workspace $workspace -CompilerExe $CompilerExe -Case $case -CaseDir $caseDir
$resultPath = Join-Path $caseDir "result.json"
$result | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $resultPath -Encoding ASCII

if ($result.Ok) {
    Write-Output "PASS $($result.Name)"
    exit 0
}

Write-Output "FAIL $($result.Name)"
Write-Output $result.Detail
exit 1
